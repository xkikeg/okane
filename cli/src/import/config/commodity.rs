use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::Decimal;
use serde::{
    de::{self, value::MapAccessDeserializer},
    {Deserialize, Serialize},
};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::import::amount::OwnedAmount;

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

    /// Default hidden fee settings, if not specified in rewrite rule.
    #[serde(default)]
    pub hidden_fee: Option<HiddenFee>,

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

impl From<AccountCommoditySpec> for AccountCommodityConfig {
    fn from(value: AccountCommoditySpec) -> Self {
        AccountCommodityConfig::Spec(value)
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
            hidden_fee: other.hidden_fee.or(self.hidden_fee),
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

/// Controls hidden fee.
/// Hidden fee is a fee that is not incurred as an explicit fee,
/// but as a spread into commodity (currency) rate advertised by the operator.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HiddenFee {
    /// Spread rate of the hidden fee. Set `None` would disable hidden rate.
    #[serde(default)]
    pub spread: Option<HiddenFeeRate>,
    /// Condition when the hidden fee applied.
    #[serde(default)]
    pub condition: Option<HiddenFeeCondition>,
}

/// Rate of the hidden fee.
#[derive(Debug, PartialEq, Eq, SerializeDisplay, DeserializeFromStr, Clone)]
pub enum HiddenFeeRate {
    /// Percent incurred for the rate.
    Percent(Decimal),
    /// Fixed amount for the rate.
    Fixed(OwnedAmount),
}

impl std::fmt::Display for HiddenFeeRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Percent(pct) => write!(f, "{}%", pct),
            Self::Fixed(fixed) => write!(f, "{} {}", fixed.value, fixed.commodity),
        }
    }
}

lazy_static! {
    static ref HIDDEN_FEE_RATE_FORMAT: Regex =
        Regex::new(r"^\s*([0-9.]+)\s*([^ 0-9.]+)\s*$").unwrap();
}

impl std::str::FromStr for HiddenFeeRate {
    type Err = HiddenFeeRateParseErr;

    fn from_str(value: &str) -> Result<Self, <Self as std::str::FromStr>::Err> {
        let cs = HIDDEN_FEE_RATE_FORMAT
            .captures(value)
            .ok_or(HiddenFeeRateParseErr::InvalidFormat)?;
        let value: Decimal = cs.get(1).map(|x| x.as_str()).unwrap_or("").parse()?;
        Ok(match cs.get(2).map(|x| x.as_str()).unwrap_or("") {
            "%" => Self::Percent(value),
            commodity => Self::Fixed(OwnedAmount {
                value,
                commodity: commodity.to_string(),
            }),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HiddenFeeRateParseErr {
    #[error("invalid decimal: {0}")]
    InvalidDecimal(#[from] rust_decimal::Error),
    #[error("invalid format: hidden fee rate must be `1.23%` or `1.23 commodity`")]
    InvalidFormat,
}

/// The condition logic when the hidden fee applied.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HiddenFeeCondition {
    /// Hidden fee is always incurred, regardles of the transaction direction
    /// (credit or debit).
    #[default]
    AlwaysIncurred,
    /// Hidden fee is only incurred on debits.
    /// On credits, instead fee is negatively applied assuming it's reimbursement.
    DebitOnly,
}

impl Merge for HiddenFee {
    fn merge(self, other: Self) -> Self {
        Self {
            spread: other.spread.or(self.spread),
            condition: other.condition.or(self.condition),
        }
    }

    fn merge_from(&mut self, other: Self) {
        *self = self.clone().merge(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use assert_matches::assert_matches;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

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
                hidden_fee: None,
                rename: hashmap! {"米ドル".to_string() => "USD".to_string()},
            }),
            AccountCommodityConfig::Spec(AccountCommoditySpec {
                primary: "CHF".to_string(),
                conversion: CommodityConversionSpec {
                    amount: Some(ConversionAmountMode::Compute),
                    rate: Some(ConversionRateMode::PriceOfSecondary),
                    commodity: None,
                    disabled: Some(false),
                },
                hidden_fee: None,
                rename: hashmap! {"米ドル".to_string() => "USD".to_string()},
            })
            .merge(AccountCommodityConfig::PrimaryCommodity("JPY".to_string()))
        );
    }

    #[test]
    fn hidden_fee_display() {
        assert_eq!("0.12%", HiddenFeeRate::Percent(dec!(0.12)).to_string());
        assert_eq!(
            "1.50 JPY",
            HiddenFeeRate::Fixed(OwnedAmount {
                value: dec!(1.50),
                commodity: "JPY".to_string()
            })
            .to_string()
        );
    }

    #[test]
    fn hidden_fee_parse() {
        let got: HiddenFeeRate = ".1%".parse().unwrap();
        assert_eq!(got, HiddenFeeRate::Percent(dec!(0.1)));
        let got: HiddenFeeRate = " 10.12 % ".parse().unwrap();
        assert_eq!(got, HiddenFeeRate::Percent(dec!(10.12)));

        let got: HiddenFeeRate = "1.50 JPY".parse().unwrap();
        assert_eq!(
            got,
            HiddenFeeRate::Fixed(OwnedAmount {
                value: dec!(1.50),
                commodity: "JPY".to_string()
            })
        );

        assert_matches!(
            HiddenFeeRate::from_str("ABC"),
            Err(HiddenFeeRateParseErr::InvalidFormat)
        );
        assert_matches!(
            HiddenFeeRate::from_str("0.12"),
            Err(HiddenFeeRateParseErr::InvalidFormat)
        );
        assert_matches!(
            HiddenFeeRate::from_str("0...12 USD"),
            Err(HiddenFeeRateParseErr::InvalidDecimal(_))
        );
    }
}
