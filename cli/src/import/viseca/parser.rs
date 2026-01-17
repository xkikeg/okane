//! Defines parser for viseca format.

use std::io::BufRead;
use std::str::FromStr;

use chrono::NaiveDate;
use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::Decimal;

use super::format::{Entry, Exchange, Fee};
use super::line_reader::{Line, LineReader};
use crate::import::amount::OwnedAmount;
use crate::import::error::{AnnotateImportError, ImportError, ImportErrorKind, IntoImportError};

lazy_static! {
    static ref TRANSACTION_LINE: Regex = Regex::new(r"^(?P<date>\d{2}.\d{2}.\d{2}) (?P<edate>\d{2}.\d{2}.\d{2}) (?P<payee>.*?)(?: (?P<currency>[A-Z]{3}) (?P<examount>[0-9.']+))? (?P<amount>[0-9.']+)(?P<neg> -)?$").unwrap();
    static ref EXCHANGE_RATE_LINE: Regex = Regex::new(r"^Exchange rate (?P<rate>[0-9.]+) of (?P<date>\d{2}.\d{2}.\d{2}) (?P<scurrency>[A-Z]{3}) (?P<samount>[0-9.']+)$").unwrap();
    static ref FEE_LINE: Regex = Regex::new(r"^(?P<credit>Credit of )?[Pp]rocessing fee (?P<percent>[0-9.]+)% (?P<fcurrency>[A-Z]{3}) (?P<famount>[0-9.']+)$").unwrap();
    static ref AIR_TAG_LINE: Regex = Regex::new(r"Air-[[:alnum:]-]+:").unwrap();
}

pub struct Parser<T: BufRead> {
    reader: LineReader<T>,
    currency: String,
}

impl<T: BufRead> Parser<T> {
    pub fn new(reader: T, currency: String) -> Parser<T> {
        Parser {
            reader: LineReader::new(reader),
            currency,
        }
    }

    pub fn parse_entry(&mut self) -> Result<Option<Entry>, ImportError> {
        let line = match self.reader.read_line()? {
            Some(l) => l,
            None => return Ok(None),
        };
        let entry_base = self.parse_transaction_line(&line).annotate(|| {
            format!(
                "failed to parse transcation line @ line {}",
                line.line_count
            )
        })?;
        // if next line is empty (EOF) or [0-9], go to next.
        self.reader.peek()?;
        if self
            .reader
            .peek_buf()
            .unwrap_or("0")
            .starts_with(|c: char| c.is_ascii_digit())
        {
            return Ok(Some(entry_base));
        }
        let line = self
            .reader
            .read_line()
            .annotate("failed to read cateogry line")?
            .into_import_err(ImportErrorKind::Internal, "category line should exist")?;
        let line_count = line.line_count;
        let category = line.value.trim().to_string();
        let exchange = match &entry_base.spent {
            Some(spent) if *spent.commodity != self.currency => {
                let line = self
                    .reader
                    .read_line()
                    .annotate("failed to read exchange line")?
                    .into_import_err(ImportErrorKind::InvalidSource, || {
                        format!("exchange line missing @ line {}", line_count)
                    })?;
                Some(self.parse_exchange(&line).annotate(|| {
                    format!("failed to parse exchange line @ line {}", line.line_count)
                })?)
            }
            _ => None,
        };
        let fee = entry_base
            .spent
            .as_ref()
            .map(|_| self.consume_fee())
            .transpose()?
            .flatten();
        self.skip_air_tags()?;
        Ok(Some(Entry {
            category,
            exchange,
            fee,
            ..entry_base
        }))
    }

    /// Parses the given line as transaction line.
    fn parse_transaction_line(&self, line: &Line) -> Result<Entry, ImportError> {
        let l = line.value.trim_end();
        let c = TRANSACTION_LINE
            .captures(l)
            .into_import_err(ImportErrorKind::InvalidSource, || {
                format!("unsupported entry line: {:?}", l)
            })?;
        let date_str = expect_name(&c, "date")?;
        let date = parse_euro_date("date", date_str)?;
        let edate_str = expect_name(&c, "edate")?;
        let edate = parse_euro_date("effective date", edate_str)?;
        let payee = expect_name(&c, "payee")?.to_string();
        let sign = c
            .name("neg")
            .map(|_| Decimal::NEGATIVE_ONE)
            .unwrap_or(Decimal::ONE);
        let spent = match c.name("currency") {
            None => None,
            Some(currency) => {
                let examount_str = expect_name(&c, "examount")?;
                let examount = parse_decimal("exchanged amount", examount_str)?;
                Some(OwnedAmount {
                    commodity: currency.as_str().to_string(),
                    value: sign * examount,
                })
            }
        };
        let amount_str = expect_name(&c, "amount")?;
        let amount = parse_decimal("amount", amount_str)?;
        Ok(Entry {
            line_count: line.line_count,
            date,
            effective_date: edate,
            payee,
            amount: amount * sign,
            category: String::new(),
            spent,
            exchange: None,
            fee: None,
        })
    }

    fn parse_exchange(&self, line: &Line) -> Result<Exchange, ImportError> {
        let c = EXCHANGE_RATE_LINE
            .captures(line.value.trim_end())
            .into_import_err(
                ImportErrorKind::InvalidSource,
                "Exchange rate ... line expected",
            )?;
        let rate = expect_name(&c, "rate")?;
        let rate = parse_decimal("rate", rate)?;
        let date = expect_name(&c, "date")?;
        let date = parse_euro_date("rate date", date)?;
        let equiv_currency = expect_name(&c, "scurrency").map(ToOwned::to_owned)?;
        let equiv_amount = expect_name(&c, "samount")?;
        let equiv_amount = parse_decimal("equivalent amount", equiv_amount)?;
        Ok(Exchange {
            rate,
            rate_date: date,
            equivalent: OwnedAmount {
                commodity: equiv_currency,
                value: equiv_amount,
            },
        })
    }

    fn consume_fee(&mut self) -> Result<Option<Fee>, ImportError> {
        self.reader.peek().annotate("failed to peek fee line")?;
        match self.reader.peek_buf() {
            Some(line) if is_fee_line(line) => (),
            _ => return Ok(None),
        };
        let line = self.reader.read_line()?.into_import_err(
            ImportErrorKind::Internal,
            "this fee line read must not fail as peek succeeded",
        )?;
        self.parse_fee(&line)
            .annotate(|| format!("failed to parse fee line @ line {}", line.line_count))
    }

    fn parse_fee(&self, line: &Line) -> Result<Option<Fee>, ImportError> {
        let c = FEE_LINE.captures(line.value.trim_end()).into_import_err(
            ImportErrorKind::InvalidSource,
            "Processing fee ... line expected",
        )?;
        let credit_applier = c
            .name("credit")
            .map(|_| Decimal::NEGATIVE_ONE)
            .unwrap_or(Decimal::ONE);
        let percent = expect_name(&c, "percent")?;
        let percent = parse_decimal("fee percent", percent)?;
        let fee_currency = expect_name(&c, "fcurrency").map(ToOwned::to_owned)?;
        let fee_amount = expect_name(&c, "famount")?;
        let fee_amount = parse_decimal("fee amount", fee_amount)?;
        Ok(Some(Fee {
            percent,
            amount: OwnedAmount {
                commodity: fee_currency,
                value: fee_amount * credit_applier,
            },
        }))
    }

    fn skip_air_tags(&mut self) -> Result<(), ImportError> {
        loop {
            self.reader.peek()?;
            match self.reader.peek_buf() {
                None => break,
                Some(l) if !AIR_TAG_LINE.is_match_at(l, 0) => break,
                _ => (),
            };
            self.reader.read_line()?;
        }
        Ok(())
    }
}

fn is_fee_line(line: &str) -> bool {
    line.starts_with("Processing fee") || line.starts_with("Credit of processing fee")
}

fn expect_name<'a>(c: &'a regex::Captures, name: &str) -> Result<&'a str, ImportError> {
    c.name(name)
        .map(|m| m.as_str())
        .into_import_err(ImportErrorKind::Internal, || {
            format!("line regex capture must have group {name}")
        })
}

fn parse_decimal(name: &str, value: &str) -> Result<Decimal, ImportError> {
    Decimal::from_str(value.replace('\'', "").as_str())
        .into_import_err(ImportErrorKind::InvalidSource, || {
            format!("failed to parse the given string {value} as a decimal for {name}")
        })
}

fn parse_euro_date(name: &str, value: &str) -> Result<NaiveDate, ImportError> {
    NaiveDate::parse_from_str(value, "%d.%m.%y")
        .into_import_err(ImportErrorKind::InvalidSource, || {
            format!("failed to parse the given string {value} as a date for {name}")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn parse_euro_date_valid() {
        assert_eq!(
            parse_euro_date("date", "22.06.20").unwrap(),
            NaiveDate::from_ymd_opt(2020, 6, 22).unwrap()
        );
    }

    #[test]
    fn parse_euro_date_invalid() {
        parse_euro_date("date", "35.06.20").unwrap_err();
    }

    #[test]
    fn parse_decimal_valid() {
        assert_eq!(parse_decimal("decimal", "1.00").unwrap(), dec!(1.00));
        assert_eq!(
            parse_decimal("decimal", "123'456'789.64").unwrap(),
            dec!(123456789.64)
        );
    }

    #[test]
    fn parse_entry_fails_invalid_transaction() {
        let input = indoc! {"
            99.99.20 11.22.33 invalid date transaction 1'234.56
        "};
        let mut p = Parser::new(input.as_bytes(), "CHF".to_string());

        let got_err = p.parse_entry().unwrap_err();

        assert_eq!(ImportErrorKind::InvalidSource, got_err.error_kind());
        assert!(
            got_err
                .message()
                .starts_with("failed to parse transcation line @ line 1"),
            "actual message: {}",
            got_err.message()
        );
        assert!(
            got_err.message().ends_with("as a date for date"),
            "actual message: {}",
            got_err.message()
        );
    }

    #[test]
    fn parse_entry_fails_missing_exchange() {
        let input = indoc! {"
            10.08.20 11.08.20 Super gas EUR 46.88 52.10
            unknown category
        "};
        let mut p = Parser::new(input.as_bytes(), "CHF".to_string());

        let got_err = p.parse_entry().unwrap_err();

        assert_eq!(ImportErrorKind::InvalidSource, got_err.error_kind());
        assert!(
            got_err
                .message()
                .starts_with("exchange line missing @ line 2"),
            "actual message: {}",
            got_err.message()
        );
    }

    #[test]
    fn parse_entry_fails_invalid_exchange() {
        let input = indoc! {"
            10.08.20 11.08.20 Super gas EUR 46.88 52.10
            unknown category
            Exchange rate invalid
        "};
        let mut p = Parser::new(input.as_bytes(), "CHF".to_string());

        let got_err = p.parse_entry().unwrap_err();

        assert_eq!(ImportErrorKind::InvalidSource, got_err.error_kind());
        assert!(
            got_err
                .message()
                .starts_with("failed to parse exchange line @ line 3"),
            "actual message: {}",
            got_err.message()
        );
    }

    #[test]
    fn parse_entry_fails_invalid_processing_fee() {
        let input = indoc! {"
            10.08.20 11.08.20 Super gas EUR 46.88 52.10
            unknown category
            Exchange rate 1.092432 of 09.08.20 CHF 51.20
            Processing fee invalid
        "};
        let mut p = Parser::new(input.as_bytes(), "CHF".to_string());

        let got_err = p.parse_entry().unwrap_err();

        assert_eq!(ImportErrorKind::InvalidSource, got_err.error_kind());
        assert!(
            got_err
                .message()
                .starts_with("failed to parse fee line @ line 4: Processing fee"),
            "actual message: {}",
            got_err.message()
        );
    }

    #[test]
    fn test_parse_entry() {
        let input = indoc! {"
            21.06.20 22.06.20 Your payment - Thank you 4'204.75 -
            22.06.20 24.06.20 foo shop 100 1'234.56
            Catering Service
            29.06.20 01.07.20 foo shop 100 56.23 -
            Catering Service
            10.08.20 11.08.20 Super gas EUR 46.88 52.10
            Service stations
            Exchange rate 1.092432 of 09.08.20 CHF 51.20
            Processing fee 1.75% CHF 0.90
            13.12.20 15.12.20 PAYPAL *STEAM GAMES, 35314369001 GB CHF 19.00 19.35
            Game, toy, and hobby shops
            Processing fee 1.75% CHF 0.35
        "};
        let mut p = Parser::new(input.as_bytes(), "CHF".to_string());
        assert_eq!(
            Some(Entry {
                line_count: 1,
                date: NaiveDate::from_ymd_opt(2020, 6, 21).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 6, 22).unwrap(),
                payee: "Your payment - Thank you".to_string(),
                amount: dec!(-4204.75),
                category: String::new(),
                spent: None,
                exchange: None,
                fee: None,
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(
            Some(Entry {
                line_count: 2,
                date: NaiveDate::from_ymd_opt(2020, 6, 22).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 6, 24).unwrap(),
                payee: "foo shop 100".to_owned(),
                amount: dec!(1234.56),
                category: "Catering Service".to_owned(),
                spent: None,
                exchange: None,
                fee: None,
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(
            Some(Entry {
                line_count: 4,
                date: NaiveDate::from_ymd_opt(2020, 6, 29).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 7, 1).unwrap(),
                payee: "foo shop 100".to_string(),
                amount: dec!(-56.23),
                category: "Catering Service".to_string(),
                spent: None,
                exchange: None,
                fee: None,
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(
            Some(Entry {
                line_count: 6,
                date: NaiveDate::from_ymd_opt(2020, 8, 10).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 8, 11).unwrap(),
                payee: "Super gas".to_string(),
                amount: dec!(52.10),
                category: "Service stations".to_string(),
                spent: Some(OwnedAmount {
                    value: dec!(46.88),
                    commodity: "EUR".to_string(),
                }),
                exchange: Some(Exchange {
                    rate: dec!(1.092432),
                    rate_date: NaiveDate::from_ymd_opt(2020, 8, 9).unwrap(),
                    equivalent: OwnedAmount {
                        value: dec!(51.20),
                        commodity: "CHF".to_string(),
                    },
                }),
                fee: Some(Fee {
                    percent: dec!(1.75),
                    amount: OwnedAmount {
                        value: dec!(0.90),
                        commodity: "CHF".to_string(),
                    },
                }),
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(
            Some(Entry {
                line_count: 10,
                date: NaiveDate::from_ymd_opt(2020, 12, 13).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 12, 15).unwrap(),
                payee: "PAYPAL *STEAM GAMES, 35314369001 GB".to_string(),
                amount: dec!(19.35),
                category: "Game, toy, and hobby shops".to_string(),
                spent: Some(OwnedAmount {
                    value: dec!(19.00),
                    commodity: "CHF".to_string(),
                }),
                exchange: None,
                fee: Some(Fee {
                    percent: dec!(1.75),
                    amount: OwnedAmount {
                        commodity: "CHF".to_string(),
                        value: dec!(0.35),
                    },
                },),
            },),
            p.parse_entry().unwrap()
        );
        assert_eq!(None, p.parse_entry().unwrap());
    }

    #[test]
    fn test_parse_entry_no_processing_fee() {
        let input = indoc! {"
            21.06.20 22.06.20 Your payment - Thank you 4'204.75 -
            10.08.20 11.08.20 Super gas EUR 46.88 52.10
            Service stations
            Exchange rate 1.092432 of 09.08.20 CHF 51.20
            20.08.20 23.08.20 MY TAXI, Amsterdam NL EUR 1.00 1.05
            Taxicabs and limousines
            Exchange rate 1.045704 of 23.08.20 CHF 1.05
        "};
        let mut p = Parser::new(input.as_bytes(), "CHF".to_string());
        assert_eq!(
            Some(Entry {
                line_count: 1,
                date: NaiveDate::from_ymd_opt(2020, 6, 21).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 6, 22).unwrap(),
                payee: "Your payment - Thank you".to_string(),
                amount: dec!(-4204.75),
                category: String::new(),
                spent: None,
                exchange: None,
                fee: None,
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(
            Some(Entry {
                line_count: 2,
                date: NaiveDate::from_ymd_opt(2020, 8, 10).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 8, 11).unwrap(),
                payee: "Super gas".to_string(),
                amount: dec!(52.10),
                category: "Service stations".to_string(),
                spent: Some(OwnedAmount {
                    value: dec!(46.88),
                    commodity: "EUR".to_string(),
                }),
                exchange: Some(Exchange {
                    rate: dec!(1.092432),
                    rate_date: NaiveDate::from_ymd_opt(2020, 8, 9).unwrap(),
                    equivalent: OwnedAmount {
                        value: dec!(51.20),
                        commodity: "CHF".to_string(),
                    },
                }),
                fee: None,
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(
            Some(Entry {
                line_count: 5,
                date: NaiveDate::from_ymd_opt(2020, 8, 20).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 8, 23).unwrap(),
                payee: "MY TAXI, Amsterdam NL".to_string(),
                amount: dec!(1.05),
                category: "Taxicabs and limousines".to_string(),
                spent: Some(OwnedAmount {
                    value: dec!(1.00),
                    commodity: "EUR".to_string(),
                }),
                exchange: Some(Exchange {
                    rate: dec!(1.045704),
                    rate_date: NaiveDate::from_ymd_opt(2020, 8, 23).unwrap(),
                    equivalent: OwnedAmount {
                        value: dec!(1.05),
                        commodity: "CHF".to_string(),
                    }
                }),
                fee: None,
            },),
            p.parse_entry().unwrap()
        );
        assert_eq!(None, p.parse_entry().unwrap());
    }

    // TODO: Support this pattern.
    // https://github.com/xkikeg/okane/issues/340
    #[test]
    #[ignore]
    fn test_parse_entry_without_category() {
        let input = indoc! {"
            16.08.23 18.08.23 MY AIR, FRANKFURT AT EUR 942.84 935.05
            Exchange rate 0.9746651982 of 16.08.23 CHF 918.95
            Processing fee 1.75% CHF 16.10
        "};

        let mut p = Parser::new(input.as_bytes(), "CHF".to_string());

        assert_eq!(
            Some(Entry {
                line_count: 1,
                date: NaiveDate::from_ymd_opt(2023, 8, 16).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2023, 8, 18).unwrap(),
                payee: "MY AIR, FRANKFURT AT".to_string(),
                amount: dec!(935.05),
                category: "Air carriers, airlines".to_string(),
                spent: Some(OwnedAmount {
                    value: dec!(942.84),
                    commodity: "EUR".to_string(),
                }),
                exchange: Some(Exchange {
                    rate: dec!(0.9746651982),
                    rate_date: NaiveDate::from_ymd_opt(2023, 8, 16).unwrap(),
                    equivalent: OwnedAmount {
                        value: dec!(918.95),
                        commodity: "CHF".to_string(),
                    },
                }),
                fee: Some(Fee {
                    percent: dec!(1.75),
                    amount: OwnedAmount {
                        commodity: "CHF".to_string(),
                        value: dec!(16.10),
                    },
                },),
            }),
            p.parse_entry().unwrap()
        );
    }

    #[test]
    fn test_parse_entry_air() {
        let input = indoc! {"
            16.08.23 18.08.23 MY AIR, FRANKFURT AT EUR 942.84 935.05
            Air carriers, airlines
            Exchange rate 0.9746651982 of 16.08.23 CHF 918.95
            Processing fee 1.75% CHF 16.10
            Air-Pass-Name: kikeg MR
            Air-Ticket-Nbr: 0123456789
            Air-Trav-Agt-Name: KIKEG AIR
            Air-Departure-Date: 230816
            Air-Origin-City: HND
            Air-Des-City: ZRH
            22.06.20 24.06.20 foo shop 100 1'234.56
            Catering Service
        "};
        let mut p = Parser::new(input.as_bytes(), "CHF".to_string());
        assert_eq!(
            Some(Entry {
                line_count: 1,
                date: NaiveDate::from_ymd_opt(2023, 8, 16).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2023, 8, 18).unwrap(),
                payee: "MY AIR, FRANKFURT AT".to_string(),
                amount: dec!(935.05),
                category: "Air carriers, airlines".to_string(),
                spent: Some(OwnedAmount {
                    value: dec!(942.84),
                    commodity: "EUR".to_string(),
                }),
                exchange: Some(Exchange {
                    rate: dec!(0.9746651982),
                    rate_date: NaiveDate::from_ymd_opt(2023, 8, 16).unwrap(),
                    equivalent: OwnedAmount {
                        value: dec!(918.95),
                        commodity: "CHF".to_string(),
                    },
                }),
                fee: Some(Fee {
                    percent: dec!(1.75),
                    amount: OwnedAmount {
                        commodity: "CHF".to_string(),
                        value: dec!(16.10),
                    },
                },),
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(
            Some(Entry {
                line_count: 11,
                date: NaiveDate::from_ymd_opt(2020, 6, 22).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 6, 24).unwrap(),
                payee: "foo shop 100".to_owned(),
                amount: dec!(1234.56),
                category: "Catering Service".to_owned(),
                spent: None,
                exchange: None,
                fee: None,
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(None, p.parse_entry().unwrap());
    }
}
