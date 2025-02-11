use std::{borrow::Cow, marker::PhantomData};

use rust_decimal::Decimal;
use serde::{
    de::value::{CowStrDeserializer, MapAccessDeserializer},
    Deserialize, Serialize,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    #[serde(rename = "BkToCstmrStmt")]
    pub bank_to_customer: BankToCustomerStatement,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BankToCustomerStatement {
    #[serde(rename = "Stmt")]
    pub statements: Vec<Statement>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Statement {
    #[serde(rename = "Bal")]
    pub balance: Vec<Balance>,
    #[serde(rename = "Ntry")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Balance {
    #[serde(rename = "Tp")]
    pub balance_type: BalanceType,
    #[serde(rename = "Amt")]
    pub amount: Amount,
    #[serde(rename = "CdtDbtInd")]
    pub credit_or_debit: CreditDebitIndicator,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BalanceType {
    #[serde(rename = "CdOrPrtry")]
    pub credit_or_property: CodeOrProperty,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeOrProperty {
    #[serde(rename = "Cd")]
    pub code: BalanceCodeValue,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BalanceCodeValue {
    #[serde(rename = "$text")]
    pub value: BalanceCode,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BalanceCode {
    #[serde(rename = "OPBD")]
    Opening,
    #[serde(rename = "CLBD")]
    Closing,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreditDebitIndicator {
    #[serde(rename = "$text")]
    pub value: CreditOrDebit,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum CreditOrDebit {
    #[serde(rename = "CRDT")]
    Credit,
    #[serde(rename = "DBIT")]
    Debit,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    #[serde(rename = "Amt")]
    pub amount: Amount,
    #[serde(rename = "CdtDbtInd")]
    pub credit_or_debit: CreditDebitIndicator,
    #[serde(rename = "BookgDt")]
    pub booking_date: DateHolder,
    #[serde(rename = "ValDt")]
    pub value_date: Option<DateHolder>,
    #[serde(rename = "BkTxCd")]
    pub bank_transaction_code: BankTransactionCode,
    #[serde(rename = "Chrgs")]
    pub charges: Option<Charges>,
    #[serde(rename = "NtryDtls", default)]
    pub details: EntryDetails,
    #[serde(rename = "AddtlNtryInf")]
    pub additional_info: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BankTransactionCode {
    #[serde(rename = "Domn")]
    pub domain: Option<Domain>,
    #[serde(rename = "Prtry")]
    pub proprietary: Option<Proprietary>,
}

impl BankTransactionCode {
    pub fn domain_code(&self) -> Option<DomainCode> {
        self.domain.as_ref().map(|x| x.code.value)
    }

    pub fn domain_family_code(&self) -> Option<DomainFamilyCode> {
        self.domain.as_ref().map(|x| x.family.code.value)
    }

    pub fn domain_sub_family_code(&self) -> Option<DomainSubFamilyCode> {
        self.domain.as_ref().map(|x| x.family.sub_family_code.value)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Domain {
    #[serde(rename = "Cd")]
    pub code: DomainCodeValue,
    #[serde(rename = "Fmly")]
    pub family: DomainFamily,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainFamily {
    #[serde(rename = "Cd")]
    pub code: DomainFamilyCodeValue,
    #[serde(rename = "SubFmlyCd")]
    pub sub_family_code: DomainSubFamilyCodeValue,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainCodeValue {
    #[serde(rename = "$text")]
    pub value: DomainCode,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum DomainCode {
    #[serde(rename = "PMNT")]
    Payment,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainFamilyCodeValue {
    #[serde(rename = "$text")]
    pub value: DomainFamilyCode,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainSubFamilyCodeValue {
    #[serde(rename = "$text")]
    pub value: DomainSubFamilyCode,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum DomainFamilyCode {
    #[serde(rename = "ICDT")]
    IssuedCreditTransfers,
    #[serde(rename = "RCDT")]
    ReceivedCreditTransfers,
    #[serde(rename = "RDDT")]
    ReceivedDirectDebits,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum DomainSubFamilyCode {
    #[serde(rename = "AUTT")]
    AutomaticTransfer,
    #[serde(rename = "DAJT")]
    DebitAdjustment,
    #[serde(rename = "PMDD")]
    PaymentDirectDebit,
    #[serde(rename = "SALA")]
    Salary,
    #[serde(rename = "STDO")]
    StandingOrder,
    #[serde(rename = "OTHR")]
    Other,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Proprietary {
    #[serde(rename = "Cd")]
    pub code: String,
    #[serde(rename = "Issr")]
    pub issuer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EntryDetails {
    #[serde(rename = "Btch", default)]
    pub batch: Batch,
    #[serde(rename = "TxDtls", default)]
    pub transactions: Vec<TransactionDetails>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Batch {
    #[serde(rename = "NbOfTxs")]
    pub number_of_transactions: usize,
    // Redundant fields.
    // #[serde(rename = "TtlAmt")]
    // pub total_amount: Amount,
    // #[serde(rename = "CdtDbtInd")]
    // pub credit_or_debit: CreditDebitIndicator,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionDetails {
    #[serde(rename = "Refs")]
    pub refs: References,
    #[serde(rename = "Amt")]
    pub amount: Amount,
    #[serde(rename = "CdtDbtInd")]
    pub credit_or_debit: CreditDebitIndicator,
    #[serde(rename = "AmtDtls")]
    pub amount_details: Option<AmountDetails>,
    #[serde(rename = "Chrgs")]
    pub charges: Option<Charges>,
    #[serde(rename = "RltdPties")]
    pub related_parties: Option<RelatedParties>,
    #[serde(rename = "RmtInf")]
    pub remittance_info: Option<RemittanceInfo>,
    #[serde(rename = "AddtlTxInf")]
    pub additional_info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemittanceInfo {
    #[serde(rename = "Ustrd")]
    pub unstructured: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct References {
    #[serde(rename = "AcctSvcrRef")]
    pub account_servicer_reference: Option<String>,
    // // may be Some("NOTPROVIDED")
    // #[serde(rename = "EndToEndId")]
    // pub end_to_end_id: String,
    // // may be Some("NOTPROVIDED") or Some("000000000")
    // #[serde(rename = "InstrId")]
    // pub instruction_id: Option<String>,
    // #[serde(rename = "TxId")]
    // pub transaction_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelatedParties {
    #[serde(rename = "Dbtr")]
    pub debtor: Option<RelatedParty>,
    #[serde(rename = "Cdtr")]
    pub creditor: Option<RelatedParty>,
    #[serde(rename = "CdtrAcct")]
    pub creditor_account: Option<Account>,
    #[serde(rename = "DbtrAcct")]
    pub debtor_account: Option<Account>,
    #[serde(rename = "UltmtDbtr")]
    pub ultimate_debtor: Option<RelatedParty>,
    #[serde(rename = "UltmtCdtr")]
    pub ultimate_creditor: Option<RelatedParty>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RelatedParty {
    Nested(Party),
    Inline(PartyDetails),
}

impl<'de> Deserialize<'de> for RelatedParty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(RelatedPartyVisitor::default())
    }
}

struct RuminateMap<'de, M>
where
    M: serde::de::MapAccess<'de>,
{
    first_key: Option<Cow<'de, str>>,
    inner: M,
}

impl<'de, M> serde::de::MapAccess<'de> for RuminateMap<'de, M>
where
    M: serde::de::MapAccess<'de>,
{
    type Error = M::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.first_key.take() {
            Some(key) => seed.deserialize(CowStrDeserializer::new(key)).map(Some),
            _ => self.inner.next_key_seed(seed),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        self.inner.next_value_seed(seed)
    }
}

#[derive(Default)]
struct RelatedPartyVisitor<'de>(PhantomData<&'de u8>);

impl<'de> serde::de::Visitor<'de> for RelatedPartyVisitor<'de> {
    type Value = RelatedParty;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Pty element containing party details, or elements of party details")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        let key: Cow<str> = map.next_key()?.ok_or_else(|| {
            <M::Error as serde::de::Error>::custom("expected Pty element or inline party details")
        })?;
        let map = RuminateMap {
            inner: map,
            first_key: Some(key),
        };
        if map.first_key.as_deref() == Some("Pty") {
            Party::deserialize(MapAccessDeserializer::new(map)).map(RelatedParty::Nested)
        } else {
            PartyDetails::deserialize(MapAccessDeserializer::new(map)).map(RelatedParty::Inline)
        }
    }
}

impl RelatedParty {
    #[cfg(test)]
    pub fn from_inner(party: PartyDetails) -> Self {
        RelatedParty::Nested(Party { party })
    }

    pub fn name(&self) -> &str {
        let details = match self {
            RelatedParty::Nested(party) => &party.party,
            RelatedParty::Inline(party) => party,
        };
        &details.name
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Party {
    #[serde(rename = "Pty")]
    party: PartyDetails,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyDetails {
    #[serde(rename = "Nm")]
    pub name: String,

    #[serde(rename = "PstlAdr")]
    pub postal_address: Option<()>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    #[serde(rename = "Id")]
    pub id: AccountIdWrapper,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountIdWrapper {
    #[serde(rename = "$value")]
    pub value: AccountId,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountId {
    #[serde(rename = "IBAN")]
    Iban(String),
    #[serde(rename = "Othr")]
    Other(OtherAccountId),
}

impl AccountId {
    pub fn as_str_id(&self) -> &str {
        match self {
            AccountId::Iban(value) => value.as_str(),
            AccountId::Other(OtherAccountId { id }) => id.as_str(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OtherAccountId {
    #[serde(rename = "Id")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Amount {
    #[serde(rename = "@Ccy")]
    pub currency: String,
    #[serde(rename = "$value")]
    pub value: Decimal,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AmountWithExchange {
    #[serde(rename = "Amt")]
    pub amount: Amount,
    #[serde(rename = "CcyXchg")]
    pub currency_exchange: Option<CurrencyExchange>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CurrencyExchange {
    #[serde(rename = "SrcCcy")]
    pub source_currency: String,
    #[serde(rename = "TrgtCcy")]
    pub target_currency: String,
    #[serde(rename = "XchgRate")]
    pub exchange_rate: ExchangeRate,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExchangeRate {
    #[serde(rename = "$value")]
    pub value: Decimal,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AmountDetails {
    // Actual passed amount.
    #[serde(rename = "InstdAmt")]
    pub instructed: AmountWithExchange,
    // Specified transaction amount, before charge deduction.
    #[serde(rename = "TxAmt")]
    pub transaction: AmountWithExchange,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Charges {
    #[serde(rename = "TtlChrgsAndTaxAmt")]
    pub total: Option<Amount>,
    #[serde(rename = "Rcrd", default)]
    pub records: Vec<ChargeRecord>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChargeRecord {
    #[serde(rename = "Amt")]
    pub amount: Amount,
    #[serde(rename = "CdtDbtInd")]
    pub credit_or_debit: CreditDebitIndicator,
    #[serde(rename = "ChrgInclInd", default)]
    pub is_charge_included: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DateHolder {
    #[serde(rename = "$value")]
    value: Date,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Date {
    #[serde(rename = "Dt")]
    Date(chrono::NaiveDate),
    #[serde(rename = "DtTm")]
    DateTime(chrono::DateTime<chrono::FixedOffset>),
}

impl DateHolder {
    /// Creates instance from NaiveDate.
    #[cfg(test)]
    pub fn from_naive_date(date: chrono::NaiveDate) -> Self {
        Self {
            value: Date::Date(date),
        }
    }

    /// Returns the naive local date for the Date.
    pub fn as_naive_date(&self) -> chrono::NaiveDate {
        self.value.as_naive_date()
    }
}

impl Date {
    /// Returns the naive local date for the Date.
    pub fn as_naive_date(&self) -> chrono::NaiveDate {
        match &self {
            Self::Date(d) => *d,
            Self::DateTime(d) => d.date_naive(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{Path, PathBuf};

    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_account_iban() {
        let input = indoc! {"
            <Account>
                <Id>
                    <IBAN>ABCD-12345-EFGHI</IBAN>
                </Id>
            </Account>
        "};
        let account: Account = quick_xml::de::from_str(input).unwrap();
        assert_eq!("ABCD-12345-EFGHI", account.id.value.as_str_id());
    }

    #[test]
    fn parse_account_other() {
        let input = indoc! {"
            <Account>
                <Id>
                    <Othr>
                        <Id>0123456789</Id>
                    </Othr>
                </Id>
            </Account>
        "};
        let account: Account = quick_xml::de::from_str(input).unwrap();
        assert_eq!("0123456789", account.id.value.as_str_id());
    }

    #[test]
    fn serialize_related_parties() {
        let input = RelatedParties {
            debtor: Some(RelatedParty::Nested(Party {
                party: PartyDetails {
                    name: "ピカチュウ".to_string(),
                    ..PartyDetails::default()
                },
            })),
            creditor: Some(RelatedParty::Inline(PartyDetails {
                name: "サトシ".to_string(),
                ..PartyDetails::default()
            })),
            creditor_account: None,
            debtor_account: None,
            ..RelatedParties::default()
        };
        let want = concat!(
            "<RelatedParties>",
            "<Dbtr><Pty><Nm>ピカチュウ</Nm><PstlAdr/></Pty></Dbtr>",
            "<Cdtr><Nm>サトシ</Nm><PstlAdr/></Cdtr>",
            "<CdtrAcct/>",
            "<DbtrAcct/>",
            "<UltmtDbtr/>",
            "<UltmtCdtr/>",
            "</RelatedParties>",
        );

        let got = quick_xml::se::to_string(&input).unwrap();

        assert_eq!(want, got);
    }

    #[test]
    fn parse_party_new() {
        let input = indoc! { "
            <Cdtr>
              <Pty>
                <Nm>北条時行</Nm>
                <PstlAdr>
                  <AdrLine>248-0006 神奈川県鎌倉市</AdrLine>
                  <AdrLine>北条執権邸</AdrLine>
                </PstlAdr>
              </Pty>
            </Cdtr>
        "};
        let party: RelatedParty = quick_xml::de::from_str(input).unwrap();
        assert_eq!("北条時行", party.name());
    }

    #[test]
    fn parse_party_old() {
        let input = indoc! { "
            <Cdtr>
              <Nm>Albert Einstein</Nm>
              <PstlAdr>
                <AdrLine>8804 Zürich</AdrLine>
              </PstlAdr>
            </Cdtr>
        "};
        let party: RelatedParty = quick_xml::de::from_str(input).unwrap();
        assert_eq!("Albert Einstein", party.name());
    }

    #[test]
    fn parse_swiss_bank_camt_file() {
        let input: PathBuf =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/testdata/import/iso_camt.xml");
        let encoded = std::fs::read(&input).expect("must read iso_camt.xml");

        let decoded: Document =
            quick_xml::de::from_reader(encoded.as_slice()).expect("must ok to parse");

        // For now we only have limited assertion as the entire message can be too large.
        assert_eq!(decoded.bank_to_customer.statements.len(), 1);
        assert_eq!(decoded.bank_to_customer.statements[0].balance.len(), 2);
        assert_eq!(decoded.bank_to_customer.statements[0].entries.len(), 10);
    }

    #[test]
    fn parse_wise_camt_file() {
        let input: PathBuf =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/testdata/import/iso_camt_wise.xml");
        let encoded = std::fs::read(&input).expect("must read iso_camt_wise.xml");

        let decoded: Document =
            quick_xml::de::from_reader(encoded.as_slice()).expect("must ok to parse");

        assert_eq!(decoded.bank_to_customer.statements.len(), 1);
        assert_eq!(decoded.bank_to_customer.statements[0].balance.len(), 1);
        assert_eq!(decoded.bank_to_customer.statements[0].entries.len(), 10);
    }
}
