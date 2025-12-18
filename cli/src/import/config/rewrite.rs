use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::commodity::CommodityConversionSpec;

/// RewriteRule specifies the rewrite rule matched against transaction.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RewriteRule {
    /// matcher for the rewrite.
    pub matcher: RewriteMatcher,

    /// Set true to leave the match pending.
    #[serde(default)]
    pub pending: bool,

    /// Payee to be set for the matched transaction.
    #[serde(default)]
    pub payee: Option<String>,

    /// Account to be set for the matched transaction.
    #[serde(default)]
    pub account: Option<String>,

    /// Commodity (currency) conversion specification.
    ///
    /// This field is only used in CSV import, and only applicable when
    /// the transcation has multi commodities. See details for [`CommodityConversionSpec`].
    #[serde(default)]
    pub conversion: Option<CommodityConversionSpec>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RewriteMatcher {
    Or(Vec<FieldMatcher>),
    Field(FieldMatcher),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct FieldMatcher {
    pub fields: HashMap<RewriteField, String>,
}

#[derive(
    Debug,
    Eq,
    Hash,
    PartialEq,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumCount,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RewriteField {
    DomainCode,
    DomainFamily,
    DomainSubFamily,
    CreditorName,
    CreditorAccountId,
    UltimateCreditorName,
    DebtorName,
    DebtorAccountId,
    UltimateDebtorName,
    RemittanceUnstructuredInfo,
    AdditionalEntryInfo,
    AdditionalTransactionInfo,
    Commodity,
    SecondaryCommodity,
    Category,
    Payee,
}
