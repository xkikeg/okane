//! Defines parser for viseca format.

use std::io::BufRead;
use std::str::FromStr;

use chrono::NaiveDate;
use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::Decimal;

use super::format::{Entry, Exchange, Fee};
use crate::import::{amount::OwnedAmount, ImportError};

lazy_static! {
    static ref FIRST_LINE: Regex = Regex::new(r"^(?P<date>\d{2}.\d{2}.\d{2}) (?P<edate>\d{2}.\d{2}.\d{2}) (?P<payee>.*?)(?: (?P<currency>[A-Z]{3}) (?P<examount>[0-9.']+))? (?P<amount>[0-9.']+)(?P<neg> -)?$").unwrap();
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
        let read_bytes = self.reader.read_line()?;
        if read_bytes == 0 {
            return Ok(None);
        };
        let l = self.reader.buf.as_str().trim_end();
        let line_count = self.reader.line_count;
        let c = FIRST_LINE
            .captures(l)
            .ok_or_else(|| self.err(format!("unsupported entry line: {:?}", l)))?;
        let entry_base = self.parse_first_line(c)?;
        let next_line_size = self.reader.peek()?;
        if next_line_size == 0 || self.reader.buf.chars().next().unwrap().is_ascii_digit() {
            return Ok(Some(Entry {
                line_count,
                ..entry_base
            }));
        }
        let read_bytes = self.reader.read_line()?;
        if read_bytes == 0 {
            return Err(self.err("category line not found"));
        }
        let category = self.reader.buf.trim().to_string();
        let exchange = entry_base
            .spent
            .as_ref()
            .filter(|spent| *spent.commodity != self.currency)
            .map(|_| self.parse_exchange())
            .transpose()?;
        let fee = entry_base
            .spent
            .as_ref()
            .map(|_| self.parse_fee())
            .transpose()?
            .flatten();
        self.skip_air_tags()?;
        Ok(Some(Entry {
            line_count,
            category,
            exchange,
            fee,
            ..entry_base
        }))
    }

    fn parse_first_line(&self, c: regex::Captures) -> Result<Entry, ImportError> {
        let date_str = self.expect_name(&c, "date", "date should exist")?;
        let date =
            parse_euro_date(date_str).map_err(|x| self.err(format!("invalid date: {}", x)))?;
        let edate_str = self.expect_name(&c, "edate", "effective date should exist")?;
        let edate = parse_euro_date(edate_str)?;
        let payee = self
            .expect_name(&c, "payee", "payee should exist")
            .map(ToOwned::to_owned)?;
        let sign = c
            .name("neg")
            .map(|_| Decimal::NEGATIVE_ONE)
            .unwrap_or(Decimal::ONE);
        let spent = match c.name("currency") {
            None => None,
            Some(currency) => {
                let examount_str = self.expect_name(
                    &c,
                    "examount",
                    "currency and examount should exist together",
                )?;
                let examount = parse_decimal(examount_str)?;
                Some(OwnedAmount {
                    commodity: currency.as_str().to_string(),
                    value: sign * examount,
                })
            }
        };
        let amount_str = self.expect_name(&c, "amount", "amount should exist")?;
        let amount = parse_decimal(amount_str)?;
        Ok(Entry {
            line_count: 0,
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

    fn parse_exchange(&mut self) -> Result<Exchange, ImportError> {
        let read_bytes = self.reader.read_line()?;
        if read_bytes == 0 {
            return Err(self.err("exchange rate line not found"));
        }
        let c = EXCHANGE_RATE_LINE
            .captures(self.reader.buf.trim_end())
            .ok_or_else(|| self.err("Exchange rate ... line expected"))?;
        let rate = self.expect_name(&c, "rate", "rate must appear")?;
        let rate = parse_decimal(rate)?;
        let date = self.expect_name(&c, "date", "date must appear")?;
        let date = parse_euro_date(date)?;
        let equiv_currency = self
            .expect_name(&c, "scurrency", "equivalent currency must appear")
            .map(ToOwned::to_owned)?;
        let equiv_amount = self.expect_name(&c, "samount", "equivalent amount must appear")?;
        let equiv_amount = parse_decimal(equiv_amount)?;
        Ok(Exchange {
            rate,
            rate_date: date,
            equivalent: OwnedAmount {
                commodity: equiv_currency,
                value: equiv_amount,
            },
        })
    }

    fn parse_fee(&mut self) -> Result<Option<Fee>, ImportError> {
        fn is_fee_prefix(val: &str) -> bool {
            val.starts_with("Processing fee") || val.starts_with("Credit of processing fee")
        }

        let next_line_size = self.reader.peek()?;
        if next_line_size == 0 || !is_fee_prefix(&self.reader.buf) {
            return Ok(None);
        }
        let read_bytes = self.reader.read_line()?;
        if read_bytes == 0 {
            return Err(self.err("Processing fee line not found"));
        }
        let c = FEE_LINE
            .captures(self.reader.buf.trim_end())
            .ok_or_else(|| self.err("Processing fee ... line expected"))?;
        let credit_applier = c
            .name("credit")
            .map(|_| Decimal::NEGATIVE_ONE)
            .unwrap_or(Decimal::ONE);
        let percent = self.expect_name(&c, "percent", "fee percent must appear")?;
        let percent = parse_decimal(percent)?;
        let fee_currency = self
            .expect_name(&c, "fcurrency", "fee currency must appear")
            .map(ToOwned::to_owned)?;
        let fee_amount = self.expect_name(&c, "famount", "fee amount must appear")?;
        let fee_amount = parse_decimal(fee_amount)?;
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
            let next_line_size = self.reader.peek()?;
            if next_line_size == 0 || !AIR_TAG_LINE.is_match_at(&self.reader.buf, 0) {
                break;
            }
            self.reader.read_line()?;
        }
        Ok(())
    }

    fn expect_name<'a>(
        &self,
        c: &'a regex::Captures,
        name: &str,
        msg: &str,
    ) -> Result<&'a str, ImportError> {
        c.name(name)
            .map(|m| m.as_str())
            .ok_or_else(|| self.err(msg))
    }

    fn err<U: AsRef<str>>(&self, msg: U) -> ImportError {
        ImportError::Viseca(format!(
            "{} @ line {}",
            msg.as_ref(),
            self.reader.line_count
        ))
    }
}

struct LineReader<T: BufRead> {
    reader: T,
    buf: String,
    // read byte
    is_peek: Option<usize>,
    line_count: usize,
}

impl<T: BufRead> LineReader<T> {
    fn new(r: T) -> LineReader<T> {
        LineReader {
            reader: r,
            buf: String::new(),
            is_peek: None,
            line_count: 0,
        }
    }

    /// Peek the next line, and returns the ref if available, None in EOF.
    /// It'll invalidate the previous buf.
    fn peek(&mut self) -> Result<usize, std::io::Error> {
        if let Some(read_bytes) = self.is_peek {
            return Ok(read_bytes);
        }
        self.buf.clear();
        let read_bytes = self.reader.read_line(&mut self.buf)?;
        self.is_peek = Some(read_bytes);
        Ok(read_bytes)
    }

    fn read_line(&mut self) -> Result<usize, ImportError> {
        if let Some(read_bytes) = self.is_peek {
            self.line_count += 1;
            self.is_peek = None;
            return Ok(read_bytes);
        }
        self.buf.clear();
        let read_bytes = self.reader.read_line(&mut self.buf)?;
        self.line_count += 1;
        Ok(read_bytes)
    }
}

fn parse_euro_date(s: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(s, "%d.%m.%y")
}

fn parse_decimal(s: &str) -> Result<Decimal, rust_decimal::Error> {
    Decimal::from_str(s.replace('\'', "").as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn line_reader_peek_and_read_line() {
        let input = indoc! {"
            First line
            Second line
            Third line
        "};
        let mut r = LineReader::new(input.as_bytes());

        assert_eq!(11, r.peek().unwrap());
        assert_eq!("First line\n", &r.buf);
        assert_eq!(0, r.line_count);
        assert_eq!(11, r.peek().unwrap());
        assert_eq!("First line\n", &r.buf);
        assert_eq!(0, r.line_count);
        assert_eq!(11, r.read_line().unwrap());
        assert_eq!("First line\n", &r.buf);
        assert_eq!(1, r.line_count);
        assert_eq!(12, r.read_line().unwrap());
        assert_eq!("Second line\n", &r.buf);
        assert_eq!(2, r.line_count);
        assert_eq!(11, r.peek().unwrap());
        assert_eq!("Third line\n", &r.buf);
        assert_eq!(2, r.line_count);
        assert_eq!(11, r.read_line().unwrap());
        assert_eq!("Third line\n", &r.buf);
        assert_eq!(3, r.line_count);
        assert_eq!(0, r.peek().unwrap());
        assert_eq!(0, r.read_line().unwrap());
    }

    #[test]
    fn parse_euro_date_valid() {
        assert_eq!(
            parse_euro_date("22.06.20").unwrap(),
            NaiveDate::from_ymd_opt(2020, 6, 22).unwrap()
        );
    }

    #[test]
    fn parse_euro_date_invalid() {
        parse_euro_date("35.06.20").unwrap_err();
    }

    #[test]
    fn parse_decimal_valid() {
        assert_eq!(parse_decimal("1.00").unwrap(), dec!(1.00));
        assert_eq!(parse_decimal("123'456'789.64").unwrap(), dec!(123456789.64));
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
