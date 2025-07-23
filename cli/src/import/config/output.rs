use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::merge::Merge;

/// Spec to describe the output formatting.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct OutputSpec {
    /// Output commodity format.
    #[serde(default)]
    pub commodity: OutputCommoditySpec,
}

impl Merge for OutputSpec {
    fn merge_from(&mut self, other: Self) {
        self.commodity.merge_from(other.commodity);
    }
}

/// Spec to describe set of commodities format styling.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct OutputCommoditySpec {
    /// Default style applied for all commodities.
    #[serde(default)]
    pub default: OutputCommodityDetailsSpec,
    /// Overrides specific to the map key commodity.
    #[serde(default)]
    pub overrides: HashMap<String, OutputCommodityDetailsSpec>,
}

impl Merge for OutputCommoditySpec {
    fn merge_from(&mut self, other: Self) {
        self.default.merge_from(other.default);
        self.overrides.merge_from(other.overrides);
    }
}

impl From<OutputCommoditySpec>
    for okane_core::utility::ConfigResolver<
        String,
        okane_core::syntax::display::CommodityDisplayOption,
    >
{
    fn from(value: OutputCommoditySpec) -> Self {
        Self::new(
            value.default.into(),
            value
                .overrides
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        )
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct OutputCommodityDetailsSpec {
    /// Format of the amount.
    pub style: Option<CommodityFormatStyle>,
    /// Scale of the amount, which is the minimal number of digits below the decimal point.
    pub scale: Option<u8>,
}

impl Merge for OutputCommodityDetailsSpec {
    fn merge_from(&mut self, other: Self) {
        *self = Self {
            style: other.style.or(self.style),
            scale: other.scale.or(self.scale),
        }
    }
}

impl From<OutputCommodityDetailsSpec> for okane_core::syntax::display::CommodityDisplayOption {
    fn from(value: OutputCommodityDetailsSpec) -> Self {
        Self {
            format: value.style.map(|x| x.into()),
            min_scale: value.scale,
        }
    }
}

/// Key represents the field abstracted way.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommodityFormatStyle {
    /// Plain number, such as 1234.567
    Plain,
    /// Comma separated number, such as 1,234.56.
    Comma3Dot,
}

impl From<CommodityFormatStyle> for pretty_decimal::Format {
    fn from(value: CommodityFormatStyle) -> Self {
        match value {
            CommodityFormatStyle::Plain => Self::Plain,
            CommodityFormatStyle::Comma3Dot => Self::Comma3Dot,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    #[test]
    fn merge_output_spec() {
        let spec1 = OutputSpec {
            commodity: OutputCommoditySpec {
                default: OutputCommodityDetailsSpec {
                    style: Some(CommodityFormatStyle::Plain),
                    scale: Some(1),
                },
                overrides: hashmap! {
                    "USD".to_string() => OutputCommodityDetailsSpec {
                        style: Some(CommodityFormatStyle::Comma3Dot),
                        scale: None,
                    },
                },
            },
        };
        let spec2 = OutputSpec {
            commodity: OutputCommoditySpec {
                default: OutputCommodityDetailsSpec {
                    style: Some(CommodityFormatStyle::Comma3Dot),
                    scale: None,
                },
                overrides: hashmap! {
                    "USD".to_string() => OutputCommodityDetailsSpec {
                        style: None,
                        scale: Some(3),
                    },
                },
            },
        };
        let merged = OutputSpec {
            commodity: OutputCommoditySpec {
                default: OutputCommodityDetailsSpec {
                    style: Some(CommodityFormatStyle::Comma3Dot),
                    scale: Some(1),
                },
                overrides: hashmap! {
                    "USD".to_string() => OutputCommodityDetailsSpec {
                        style: Some(CommodityFormatStyle::Comma3Dot),
                        scale: Some(3),
                    },
                },
            },
        };

        assert_eq!(merged, spec1.merge(spec2));
    }
}
