use std::collections::HashMap;

use serde::{
    de::{self, value::MapAccessDeserializer},
    Deserialize, Serialize,
};

use crate::one_based::OneBasedIndex;

use super::merge::{non_empty_right_most, Merge};

/// FormatSpec describes the several format used in import target.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FormatSpec {
    /// Specify the date format, in [chrono::format::strftime] compatible format.
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
        let date = non_empty_right_most!(self.date, other.date);
        let fields = non_empty_right_most!(self.fields, other.fields);
        let delimiter = non_empty_right_most!(self.delimiter, other.delimiter);
        let skip = other.skip.or(self.skip);
        let row_order = other.row_order.or(self.row_order);
        Self {
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
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKey {
    /// Date of the transaction.
    Date,
    /// Payee, opposite side of the transaction.
    Payee,
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
    Index(OneBasedIndex),
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

impl<'de> de::Visitor<'de> for FieldPosVisitor {
    type Value = FieldPos;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a positive integer, a string or a map specifying a template"
        )
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match OneBasedIndex::from_one_based(v as usize) {
            Ok(x) => Ok(FieldPos::Index(x)),
            Err(_) => Err(E::invalid_value(
                de::Unexpected::Unsigned(v),
                &"a positive column index",
            )),
        }
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
