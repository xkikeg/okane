use std::collections::HashMap;

use one_based::OneBasedU32;
use serde::{
    Deserialize, Serialize,
    de::{self, value::MapAccessDeserializer},
};

use super::merge::{Merge, merge_non_empty};

/// FormatSpec describes the several format used in import target.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FormatSpec {
    /// File type. If the field is not set, we'll guess from the suffix for the following files.
    ///
    /// * `.csv`: CSV
    /// * `.tsv`: TSV (CSV with tab delimiter)
    ///
    /// For others, it's mandatory to set this field.
    #[serde(default)]
    pub file_type: Option<FileType>,
    /// Specify the date format, in [`chrono::format::strftime`] compatible format.
    #[serde(default)]
    pub date: String,
    /// Mapping from abstracted field key to abstracted position.
    #[serde(default)]
    pub fields: HashMap<FieldKey, FieldPos>,
    /// Delimiter for the CSV. Leave it empty to use default ",".
    #[serde(default)]
    pub delimiter: String,
    pub skip: Option<SkipSpec>,
    /// Order of the row. By default (if None), old to new order.
    pub row_order: Option<RowOrder>,
}

impl Merge for FormatSpec {
    fn merge(self, other: Self) -> Self {
        let date = merge_non_empty!(self.date, other.date);
        let fields = merge_non_empty!(self.fields, other.fields);
        let delimiter = merge_non_empty!(self.delimiter, other.delimiter);
        let skip = other.skip.or(self.skip);
        let row_order = other.row_order.or(self.row_order);
        Self {
            file_type: other.file_type.or(self.file_type),
            date,
            fields,
            delimiter,
            skip,
            row_order,
        }
    }

    fn merge_from(&mut self, other: Self) {
        // insufficient implementation, but safer unless we have derive macro.
        *self = self.clone().merge(other)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileType {
    /// CSV (comma separated values) file.
    Csv,
    /// TSV (tab separated values) file, which is equivalent to CSV with `\t` delimiter.
    Tsv,
    /// ISO Camt053 format.
    IsoCamt053,
    /// Viseca format.
    Viseca,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct SkipSpec {
    /// The number of lines skipped at head.
    /// Only supported for CSV.
    pub head: i32,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum RowOrder {
    #[default]
    OldToNew,
    NewToOld,
}

/// Key represents the field abstracted way.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FieldKey {
    /// Date of the transaction.
    Date,
    /// Payee, opposite side of the transaction.
    Payee,
    /// Transaction code which can uniquely identify the transaction.
    Code,
    /// Category of the transaction.
    Category,
    /// Side note.
    Note,
    /// Amount of the transcation, which can be positive or negative.
    Amount,
    /// Amount increasing the total balance.
    Credit,
    /// Amount decreasing the total balance.
    Debit,
    /// Remaining balance amount.
    Balance,
    /// Currency (commodity) of the transaction.
    /// If not specified, fallback to primary commodity specified in the account.
    Commodity,
    /// Currency rate used in the statement.
    Rate,
    /// Secondary amount represents the amount in the secondary currency.
    /// Useful when the transaction exchanges amount into another commodity into the other.
    /// Detailed logic could be referred in [CommodityConversionSpec].
    SecondaryAmount,
    /// Secondary commodity corresponding the [FieldKey::SecondaryAmount].
    /// Also refer [CommodityConversionSpec] for the detialed explanation.
    SecondaryCommodity,
    /// Charge, commision or fee related to the transaction.
    Charge,
}

#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub enum FieldPos {
    Index(OneBasedU32),
    Label(String),
    Template(TemplateField),
}

impl<'de> Deserialize<'de> for FieldPos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(FieldPosVisitor)
    }
}

struct FieldPosVisitor;

const POS_ERR_MSG: &str = "a 32-bit unsigned positive index";

impl<'de> de::Visitor<'de> for FieldPosVisitor {
    type Value = FieldPos;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a positive integer, a string or a map specifying a template"
        )
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let v: u64 = v
            .try_into()
            .map_err(|_| E::invalid_value(de::Unexpected::Signed(v), &POS_ERR_MSG))?;
        self.visit_u64(v)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let v: u32 = v
            .try_into()
            .map_err(|_| E::invalid_value(de::Unexpected::Unsigned(v), &POS_ERR_MSG))?;
        OneBasedU32::from_one_based(v)
            .map(FieldPos::Index)
            .ok_or_else(|| E::invalid_value(de::Unexpected::Unsigned(v.into()), &POS_ERR_MSG))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(FieldPos::Label(v))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        TemplateField::deserialize(MapAccessDeserializer::new(map)).map(FieldPos::Template)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TemplateField {
    pub template: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use crate::one_based_macro::one_based_32;

    #[test]
    fn merge_format_spec() {
        let spec1 = FormatSpec {
            file_type: Some(FileType::Tsv),
            date: "%Y/%m/%d".to_string(),
            fields: hashmap! {
                FieldKey::Date => FieldPos::Index(one_based_32!(3)),
            },
            delimiter: ";".to_string(),
            skip: Some(SkipSpec { head: 2 }),
            row_order: Some(RowOrder::OldToNew),
        };
        let spec2 = FormatSpec {
            file_type: Some(FileType::Csv),
            date: "%Y-%m-%d".to_string(),
            fields: hashmap! {
                FieldKey::Payee => FieldPos::Index(one_based_32!(4)),
            },
            delimiter: ",".to_string(),
            skip: Some(SkipSpec { head: 3 }),
            row_order: Some(RowOrder::NewToOld),
        };

        assert_eq!(spec1, spec1.clone().merge(FormatSpec::default()));
        assert_eq!(spec1, FormatSpec::default().merge(spec1.clone()));
        // both filled, then other takes precedence
        assert_eq!(spec2, spec1.merge(spec2.clone()));
    }

    mod deserialize_field_pos {
        use super::*;

        use serde_test::{Token, assert_de_tokens, assert_de_tokens_error};

        #[test]
        fn positive_integer_succeeds() {
            assert_de_tokens(&FieldPos::Index(one_based_32!(1)), &[Token::U16(1)]);
            assert_de_tokens(&FieldPos::Index(one_based_32!(2)), &[Token::I32(2)]);
        }

        #[test]
        fn non_positive_integer_fails() {
            assert_de_tokens_error::<FieldPos>(
                &[Token::U16(0)],
                "invalid value: integer `0`, expected a 32-bit unsigned positive index",
            );
            assert_de_tokens_error::<FieldPos>(
                &[Token::I32(-1)],
                "invalid value: integer `-1`, expected a 32-bit unsigned positive index",
            );
            assert_de_tokens_error::<FieldPos>(
                &[Token::U64(u64::MAX)],
                "invalid value: integer `18446744073709551615`, expected a 32-bit unsigned positive index",
            );
        }

        #[test]
        fn str_succeeds() {
            assert_de_tokens(&FieldPos::Label("foo".to_string()), &[Token::Str("foo")]);
        }

        #[test]
        fn map_succeeds_with_valid_key() {
            assert_de_tokens(
                &FieldPos::Template(TemplateField {
                    template: "foo".to_string(),
                }),
                &[
                    Token::Map { len: None },
                    Token::Str("template"),
                    Token::Str("foo"),
                    Token::MapEnd,
                ],
            );
        }
    }
}
