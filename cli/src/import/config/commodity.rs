use std::collections::HashMap;

use serde::{
    de::{self, value::MapAccessDeserializer},
    {Deserialize, Serialize},
};

use super::merge::{merge_non_empty, Merge};

/// CommodityConfig contains either primary commodity string, or more complex CommoditySpec.
#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
#[serde(untagged)]
pub(super) enum AccountCommodityConfig {
    PrimaryCommodity(String),
    Spec(AccountCommoditySpec),
}

impl Merge for AccountCommodityConfig {
    fn merge_from(&mut self, other: Self) {
        let other: AccountCommoditySpec = other.into();
        match self {
            Self::PrimaryCommodity(_) => {
                *self = {
                    let spec: AccountCommoditySpec = self.clone().into();
                    Self::Spec(spec.merge(other))
                }
            }
            Self::Spec(x) => x.merge_from(other),
        }
    }
}

impl<'de> Deserialize<'de> for AccountCommodityConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AccountCommodityConfigVisitor)
    }
}

struct AccountCommodityConfigVisitor;

impl<'de> de::Visitor<'de> for AccountCommodityConfigVisitor {
    type Value = AccountCommodityConfig;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string or a map specifying a AccountCommoditySpec"
        )
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
        Ok(AccountCommodityConfig::PrimaryCommodity(v))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        AccountCommoditySpec::deserialize(MapAccessDeserializer::new(map))
            .map(AccountCommodityConfig::Spec)
    }
}

/// CommoditySpec describes commodity configs.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AccountCommoditySpec {
    /// Primary commodity used in the account.
    pub primary: String,
    /// Default conversion applied to all transaction, if not specified in rewrite rules.
    #[serde(default)]
    pub conversion: CommodityConversionSpec,
    /// Rename the key commodity into value.
    #[serde(default)]
    pub rename: HashMap<String, String>,
}

impl From<AccountCommodityConfig> for AccountCommoditySpec {
    fn from(value: AccountCommodityConfig) -> Self {
        match value {
            AccountCommodityConfig::PrimaryCommodity(c) => AccountCommoditySpec {
                primary: c,
                ..AccountCommoditySpec::default()
            },
            AccountCommodityConfig::Spec(spec) => spec,
        }
    }
}

impl Merge for AccountCommoditySpec {
    fn merge(self, other: Self) -> Self {
        let mut rename = self.rename;
        for (k, v) in other.rename.into_iter() {
            rename.insert(k, v);
        }
        Self {
            primary: merge_non_empty!(self.primary, other.primary),
            conversion: self.conversion.merge(other.conversion),
            rename,
        }
    }

    fn merge_from(&mut self, other: Self) {
        *self = self.clone().merge(other)
    }
}

/// Specify the currency conversion details described in the transaction.
///
/// This is useful when CSV has non-straightforward logic.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct CommodityConversionSpec {
    /// Decides how the `secondary_amount` is computed.
    pub amount: Option<ConversionAmountMode>,
    /// Overrides `secondary_commodity` with the given value.
    pub commodity: Option<String>,
    /// Decides `rate` meaning.
    pub rate: Option<ConversionRateMode>,
    /// Disable all conversions.
    pub disabled: Option<bool>,
}

impl Merge for CommodityConversionSpec {
    fn merge(self, other: Self) -> Self {
        Self {
            amount: other.amount.or(self.amount),
            commodity: other.commodity.or(self.commodity),
            rate: other.rate.or(self.rate),
            disabled: other.disabled.or(self.disabled),
        }
    }

    fn merge_from(&mut self, other: Self) {
        *self = self.clone().merge(other)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ConversionAmountMode {
    /// Extracts the `secondary_amount` from the input data field.
    #[default]
    Extract,
    /// Computes the `secondary_amount` using the specified rate.
    Compute,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ConversionRateMode {
    /// Given rate is a price of secondary commodity, e.g.
    /// `1 $secondary_commodity == $rate $commodity`.
    #[default]
    PriceOfSecondary,
    /// Given rate is a price of primary commodity, e.g.
    /// `1 $commodity == $rate $secondary_commodity`
    PriceOfPrimary,
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    #[test]
    fn merge_commodity_config() {
        // merge doesn't preserve PrimaryCommodity enum.
        assert_eq!(
            AccountCommodityConfig::Spec(AccountCommoditySpec {
                primary: "JPY".to_string(),
                ..AccountCommoditySpec::default()
            }),
            AccountCommodityConfig::PrimaryCommodity("CHF".to_string())
                .merge(AccountCommodityConfig::PrimaryCommodity("JPY".to_string()))
        );

        // merge treats PrimaryCommodity just as a Spec with primary field filled.
        assert_eq!(
            AccountCommodityConfig::Spec(AccountCommoditySpec {
                primary: "JPY".to_string(),
                conversion: CommodityConversionSpec {
                    amount: Some(ConversionAmountMode::Compute),
                    rate: Some(ConversionRateMode::PriceOfSecondary),
                    commodity: None,
                    disabled: Some(false),
                },
                rename: hashmap! {"米ドル".to_string() => "USD".to_string()},
                ..AccountCommoditySpec::default()
            }),
            AccountCommodityConfig::Spec(AccountCommoditySpec {
                primary: "CHF".to_string(),
                conversion: CommodityConversionSpec {
                    amount: Some(ConversionAmountMode::Compute),
                    rate: Some(ConversionRateMode::PriceOfSecondary),
                    commodity: None,
                    disabled: Some(false),
                },
                rename: hashmap! {"米ドル".to_string() => "USD".to_string()},
                ..AccountCommoditySpec::default()
            })
            .merge(AccountCommodityConfig::PrimaryCommodity("JPY".to_string()))
        );
    }
}
