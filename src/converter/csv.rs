pub struct CSVConverter {}

use super::ConvertError;
use crate::data;
use data::parse_comma_decimal;

use chrono::NaiveDate;
use log::warn;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

impl super::Converter for CSVConverter {
    fn convert<R: std::io::Read>(&self, r: &mut R) -> Result<Vec<data::Transaction>, ConvertError> {
        let mut res = Vec::new();
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b',')
            .from_reader(r);
        for may_record in rdr.records() {
            let r = may_record?;
            if r.len() != 6 {
                return Err(ConvertError::Other("unexpected csv length".to_string()));
            }
            let date = NaiveDate::parse_from_str(r.get(0).unwrap(), "%Y年%m月%d日")?;
            let payee = r.get(1).unwrap();
            // Ignore note at [2]
            let has_credit = r.get(3).unwrap() != "";
            let has_debit = r.get(4).unwrap() != "";
            let balance = parse_comma_decimal(r.get(5).unwrap())?;
            let mut posts = Vec::new();
            if has_credit {
                warn!("{:#?} {:#?}", r.get(3), r.get(3).unwrap().replace(',', ""));
                let credit: Decimal = parse_comma_decimal(r.get(3).unwrap()).unwrap_or(dec!(9.99));
                posts.push(data::Post{
                    account: "Incomes:Unknown".to_string(),
                    amount: data::Amount{value: -credit, commodity: "JPY".to_string()},
                    balance: None,
                });
                posts.push(data::Post{
                    account: "Assets:Banks:MyBank".to_string(),
                    amount: data::Amount{value: credit, commodity: "JPY".to_string()},
                    balance: Some(data::Amount{value: balance, commodity: "JPY".to_string()}),
                });
            } else if has_debit {
                warn!("{:#?} {:#?}", r.get(4), r.get(4).unwrap().replace(',', ""));
                let debit: Decimal = parse_comma_decimal(r.get(4).unwrap()).unwrap_or(dec!(9.99));
                posts.push(data::Post{
                    account: "Assets:Banks:MyBank".to_string(),
                    amount: data::Amount{value: -debit, commodity: "JPY".to_string()},
                    balance: Some(data::Amount{value: balance, commodity: "JPY".to_string()}),
                });
                posts.push(data::Post{
                    account: "Expenses:Unknown".to_string(),
                    amount: data::Amount{value: debit, commodity: "JPY".to_string()},
                    balance: None,
                });
            } else {
                // warning log or error?
                return Err(ConvertError::Other("credit and debit both zero".to_string()));
            }
            res.push(data::Transaction{date: date, payee: payee.to_string(), posts: posts});
        }
        return Ok(res);
    }
}
