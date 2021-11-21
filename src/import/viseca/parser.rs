use super::format::{Entry, Exchange};
use crate::import::ImportError;

use std::io::BufRead;
use std::str::FromStr;

use chrono::NaiveDate;
use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::Decimal;

lazy_static! {
    static ref FIRST_LINE: Regex = Regex::new(r"^(?P<date>\d{2}.\d{2}.\d{2}) (?P<edate>\d{2}.\d{2}.\d{2}) (?P<payee>.*?)(?: (?P<currency>[A-Z]{3}) (?P<examount>[0-9.']+))? (?P<amount>[0-9.']+)$").unwrap();
    static ref EXCHANGE_RATE_LINE: Regex = Regex::new(r"^Exchange rate (?P<rate>[0-9.]+) of (?P<date>\d{2}.\d{2}.\d{2}) (?P<scurrency>[A-Z]{3}) (?P<samount>[0-9.']+)$").unwrap();
    static ref FEE_LINE: Regex = Regex::new(r"^Processing fee (?P<percent>[0-9.]+)% (?P<fcurrency>[A-Z]{3}) (?P<famount>[0-9.']+)$").unwrap();
}

pub struct Parser<T: BufRead> {
    reader: T,
    buf: String,
    line_count: usize,
}

impl<T: BufRead> Parser<T> {
    pub fn new(reader: T) -> Parser<T> {
        Parser {
            reader,
            buf: String::new(),
            line_count: 0,
        }
    }

    pub fn parse_entry(&mut self) -> Result<Option<Entry>, ImportError> {
        let read_bytes = self.read_line()?;
        if read_bytes == 0 {
            return Ok(None);
        }
        let line_count = self.line_count;
        let l = self.buf.trim_end();
        let c = FIRST_LINE
            .captures(l)
            .ok_or_else(|| self.err(format!("unsupported entry line: {}", l)))?;
        let date_str = self.expect_name(&c, "date", "date should exist")?;
        let date = parse_euro_date(date_str)
            .map_err(|x| self.err(format!("invalid date: {}", x.to_string())))?;
        let edate_str = self.expect_name(&c, "edate", "effective date should exist")?;
        let edate = parse_euro_date(edate_str)?;
        let payee = self
            .expect_name(&c, "payee", "payee should exist")
            .map(ToOwned::to_owned)?;
        let exchange = match c.name("currency") {
            None => None,
            Some(currency) => {
                let examount_str = self.expect_name(
                    &c,
                    "examount",
                    "currency and examount should exist together",
                )?;
                let examount = parse_decimal(examount_str)?;
                Some((currency.as_str().to_string(), examount))
            }
        };
        let amount_str = self.expect_name(&c, "amount", "amount should exist")?;
        let amount = parse_decimal(amount_str)?;
        let read_bytes = self.read_line()?;
        if read_bytes == 0 {
            return Err(self.err("category line not found"));
        }
        let category = self.buf.trim().to_string();
        let exchange = exchange
            .map(|(excurrency, examount)| self.parse_exchange(excurrency, examount))
            .transpose()?;
        Ok(Some(Entry {
            line_count,
            date,
            effective_date: edate,
            payee,
            amount,
            category,
            exchange,
        }))
    }

    fn parse_exchange(
        &mut self,
        exchanged_currency: String,
        exchanged_amount: Decimal,
    ) -> Result<Exchange, ImportError> {
        let read_bytes = self.read_line()?;
        if read_bytes == 0 {
            return Err(self.err("exchange rate line not found"));
        }
        let c = EXCHANGE_RATE_LINE
            .captures(self.buf.trim_end())
            .ok_or_else(|| self.err("Exchange rate ... line expected"))?;
        let rate = self.expect_name(&c, "rate", "rate must appear")?;
        let rate = parse_decimal(rate)?;
        let date = self.expect_name(&c, "date", "date must appear")?;
        let date = parse_euro_date(date)?;
        let spent_currency = self
            .expect_name(&c, "scurrency", "spent currency must appear")
            .map(ToOwned::to_owned)?;
        let spent_amount = self.expect_name(&c, "samount", "spent amount must appear")?;
        let spent_amount = parse_decimal(spent_amount)?;
        let read_bytes = self.read_line()?;
        if read_bytes == 0 {
            return Err(self.err("fee line not found"));
        }
        let c = FEE_LINE
            .captures(self.buf.trim_end())
            .ok_or_else(|| self.err("Processing fee ... line expected"))?;
        let fee_percent = self.expect_name(&c, "percent", "fee percent must appear")?;
        let fee_percent = parse_decimal(fee_percent)?;
        let fee_currency = self
            .expect_name(&c, "fcurrency", "fee currency must appear")
            .map(ToOwned::to_owned)?;
        let fee_amount = self.expect_name(&c, "famount", "fee amount must appear")?;
        let fee_amount = parse_decimal(fee_amount)?;
        Ok(Exchange {
            rate,
            rate_date: date,
            exchanged_currency,
            exchanged_amount,
            spent_currency,
            spent_amount,
            fee_percent,
            fee_currency,
            fee_amount,
        })
    }

    fn read_line(&mut self) -> Result<usize, ImportError> {
        self.buf.clear();
        let read_bytes = self.reader.read_line(&mut self.buf)?;
        self.line_count += 1;
        Ok(read_bytes)
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
        ImportError::Viseca(format!("{} @ line {}", msg.as_ref(), self.line_count))
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
    fn parse_euro_date_valid() {
        assert_eq!(
            parse_euro_date("22.06.20").unwrap(),
            NaiveDate::from_ymd(2020, 6, 22)
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
            22.06.20 13.09.21 foo shop 100 1'234.56
            Catering Service
            10.08.20 11.08.20 Super gas EUR 46.88 52.10
            Service stations
            Exchange rate 1.092432 of 09.08.20 CHF 51.20
            Processing fee 1.75% CHF 0.90
        "};
        let mut p = Parser::new(input.as_bytes());
        assert_eq!(
            Some(Entry {
                line_count: 1,
                date: NaiveDate::from_ymd(2020, 6, 22),
                effective_date: NaiveDate::from_ymd(2021, 9, 13),
                payee: "foo shop 100".to_owned(),
                amount: dec!(1234.56),
                category: "Catering Service".to_owned(),
                exchange: None,
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(
            Some(Entry {
                line_count: 3,
                date: NaiveDate::from_ymd_opt(2020, 8, 10).unwrap(),
                effective_date: NaiveDate::from_ymd_opt(2020, 8, 11).unwrap(),
                payee: "Super gas".to_string(),
                amount: dec!(52.10),
                category: "Service stations".to_string(),
                exchange: Some(Exchange {
                    exchanged_amount: dec!(46.88),
                    exchanged_currency: "EUR".to_string(),
                    rate: dec!(1.092432),
                    rate_date: NaiveDate::from_ymd_opt(2020, 8, 9).unwrap(),
                    spent_amount: dec!(51.20),
                    spent_currency: "CHF".to_string(),
                    fee_percent: dec!(1.75),
                    fee_amount: dec!(0.90),
                    fee_currency: "CHF".to_string(),
                }),
            }),
            p.parse_entry().unwrap()
        );
        assert_eq!(None, p.parse_entry().unwrap());
    }
}
