use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Document {
    #[serde(rename = "BkToCstmrStmt")]
    pub bank_to_customer: BankToCustomerStatement,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BankToCustomerStatement {
    #[serde(rename = "Stmt")]
    pub statements: Vec<Statement>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Statement {
    #[serde(rename = "Bal")]
    pub balance: Vec<Balance>,
    #[serde(rename = "Ntry")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Balance {
    #[serde(rename = "Tp")]
    pub balance_type: BalanceType,
    #[serde(rename = "Amt")]
    pub amount: Amount,
    #[serde(rename = "CdtDbtInd")]
    pub credit_or_debit: CreditDebitIndicator,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BalanceType {
    #[serde(rename = "CdOrPrtry")]
    pub credit_or_property: CodeOrProperty,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CodeOrProperty {
    #[serde(rename = "Cd")]
    pub code: BalanceCodeValue,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BalanceCodeValue {
    #[serde(rename = "$text")]
    pub value: BalanceCode,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum BalanceCode {
    #[serde(rename = "OPBD")]
    Opening,
    #[serde(rename = "CLBD")]
    Closing,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CreditDebitIndicator {
    #[serde(rename = "$text")]
    pub value: CreditOrDebit,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum CreditOrDebit {
    #[serde(rename = "CRDT")]
    Credit,
    #[serde(rename = "DBIT")]
    Debit,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Entry {
    #[serde(rename = "Amt")]
    pub amount: Amount,
    #[serde(rename = "CdtDbtInd")]
    pub credit_or_debit: CreditDebitIndicator,
    #[serde(rename = "BookgDt")]
    pub booking_date: Date,
    #[serde(rename = "ValDt")]
    pub value_date: Date,
    #[serde(rename = "BkTxCd")]
    pub bank_transaction_code: BankTransactionCode,
    #[serde(rename = "Chrgs")]
    pub charges: Option<Charges>,
    #[serde(rename = "NtryDtls")]
    pub details: EntryDetails,
    #[serde(rename = "AddtlNtryInf")]
    pub additional_info: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BankTransactionCode {
    #[serde(rename = "Domn")]
    pub domain: Domain,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Domain {
    #[serde(rename = "Cd")]
    pub code: DomainCodeValue,
    #[serde(rename = "Fmly")]
    pub family: DomainFamily,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DomainFamily {
    #[serde(rename = "Cd")]
    pub code: DomainFamilyCodeValue,
    #[serde(rename = "SubFmlyCd")]
    pub sub_family_code: DomainSubFamilyCodeValue,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DomainCodeValue {
    #[serde(rename = "$text")]
    pub value: DomainCode,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum DomainCode {
    #[serde(rename = "PMNT")]
    Payment,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DomainFamilyCodeValue {
    #[serde(rename = "$text")]
    pub value: DomainFamilyCode,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct DomainSubFamilyCodeValue {
    #[serde(rename = "$text")]
    pub value: DomainSubFamilyCode,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum DomainFamilyCode {
    #[serde(rename = "ICDT")]
    IssuedCreditTransfers,
    #[serde(rename = "RCDT")]
    ReceivedCreditTransfers,
    #[serde(rename = "RDDT")]
    ReceivedDirectDebits,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum DomainSubFamilyCode {
    #[serde(rename = "AUTT")]
    AutomaticTransfer,
    #[serde(rename = "PMDD")]
    PaymentDirectDebit,
    #[serde(rename = "SALA")]
    Salary,
    #[serde(rename = "STDO")]
    StandingOrder,
    #[serde(rename = "OTHR")]
    Other,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct EntryDetails {
    #[serde(rename = "Btch", default)]
    pub batch: Batch,
    #[serde(rename = "TxDtls", default)]
    pub transactions: Vec<TransactionDetails>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Default)]
pub struct Batch {
    #[serde(rename = "NbOfTxs")]
    pub number_of_transactions: usize,
    // Redundant fields.
    // #[serde(rename = "TtlAmt")]
    // pub total_amount: Amount,
    // #[serde(rename = "CdtDbtInd")]
    // pub credit_or_debit: CreditDebitIndicator,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RemittanceInfo {
    #[serde(rename = "Ustrd")]
    pub unstructured: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct RelatedParties {
    #[serde(rename = "Dbtr")]
    pub debtor: Option<Party>,
    #[serde(rename = "Cdtr")]
    pub creditor: Option<Party>,
    #[serde(rename = "CdtrAcct")]
    pub creditor_account: Option<Account>,
    #[serde(rename = "DbtrAcct")]
    pub debtor_account: Option<Account>,
    #[serde(rename = "UltmtDbtr")]
    pub ultimate_debtor: Option<Party>,
    #[serde(rename = "UltmtCdtr")]
    pub ultimate_creditor: Option<Party>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Party {
    #[serde(rename = "Nm")]
    pub name: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Account {
    #[serde(rename = "Id")]
    pub id: AccountIdWrapper,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AccountIdWrapper {
    #[serde(rename = "$value")]
    pub value: AccountId,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct OtherAccountId {
    #[serde(rename = "Id")]
    pub id: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Amount {
    #[serde(rename = "@Ccy")]
    pub currency: String,
    #[serde(rename = "$value")]
    pub value: Decimal,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AmountWithExchange {
    #[serde(rename = "Amt")]
    pub amount: Amount,
    #[serde(rename = "CcyXchg")]
    pub currency_exchange: Option<CurrencyExchange>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CurrencyExchange {
    #[serde(rename = "SrcCcy")]
    pub source_currency: String,
    #[serde(rename = "TrgtCcy")]
    pub target_currency: String,
    #[serde(rename = "XchgRate")]
    pub exchange_rate: ExchangeRate,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ExchangeRate {
    #[serde(rename = "$value")]
    pub value: Decimal,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AmountDetails {
    // Actual passed amount.
    #[serde(rename = "InstdAmt")]
    pub instructed: AmountWithExchange,
    // Specified transaction amount, before charge deduction.
    #[serde(rename = "TxAmt")]
    pub transaction: AmountWithExchange,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Charges {
    #[serde(rename = "TtlChrgsAndTaxAmt")]
    pub total: Option<Amount>,
    #[serde(rename = "Rcrd")]
    pub records: Vec<ChargeRecord>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct ChargeRecord {
    #[serde(rename = "Amt")]
    pub amount: Amount,
    #[serde(rename = "CdtDbtInd")]
    pub credit_or_debit: CreditDebitIndicator,
    #[serde(rename = "ChrgInclInd", default)]
    pub is_charge_included: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Date {
    #[serde(rename = "Dt")]
    pub date: chrono::NaiveDate,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
