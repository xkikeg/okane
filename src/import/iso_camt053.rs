use quick_xml::de::DeError;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Document {
    #[serde(rename = "BkToCstmrStmt")]
    bank_to_customer: BankToCustomerStatement,
}

#[derive(Debug, Deserialize, PartialEq)]
struct BankToCustomerStatement {
    #[serde(rename = "Stmt")]
    statements: Vec<Statement>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Statement {
    #[serde(rename = "Ntry")]
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Entry {
    #[serde(rename = "Amt")]
    amount: Amount,
    // CRDT or DBIT
    // TODO: Use Enum once quick-xml serde integration fixed.
    #[serde(rename = "CdtDbtInd")]
    is_credit: String,
    #[serde(rename = "BookgDt")]
    booking_date: Date,
    #[serde(rename = "ValDt")]
    value_date: Date,
    #[serde(rename = "Chrgs")]
    charges: Option<Charges>,
    #[serde(rename = "NtryDtls")]
    details: EntryDetails,
}

#[derive(Debug, Deserialize, PartialEq)]
struct EntryDetails {
    #[serde(rename = "TxDtls")]
    transaction: TransactionDetails,
}

#[derive(Debug, Deserialize, PartialEq)]
struct TransactionDetails {
    #[serde(rename = "Refs")]
    refs: References,
    #[serde(rename = "AmtDtls")]
    amount_details: AmountDetails,
    #[serde(rename = "Chrgs")]
    charges: Option<Charges>,
    #[serde(rename = "RltdPties")]
    related_parties: RelatedParties,
    #[serde(rename = "RmtInf")]
    remittance_info: RemittanceInfo,
}

#[derive(Debug, Deserialize, PartialEq)]
struct RemittanceInfo {
    #[serde(rename = "Ustrd")]
    unstructured: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct References {
    #[serde(rename = "AcctSvcrRef")]
    account_servicer_reference: String,
    // may be Some("NOTPROVIDED")
    #[serde(rename = "EndToEndId")]
    end_to_end_id: String,
    // may be Some("NOTPROVIDED")
    #[serde(rename = "InstrId")]
    instruction_id: Option<String>,
    #[serde(rename = "TxId")]
    transaction_id: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct RelatedParties {
    #[serde(rename = "Dbtr")]
    debtor: Party,
    #[serde(rename = "Cdtr")]
    creditor: Party,
    #[serde(rename = "UltmtDbtr")]
    ultimate_debtor: Option<Party>,
    #[serde(rename = "UltmtCdtr")]
    ultimate_creditor: Option<Party>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Party {
    #[serde(rename = "Nm")]
    name: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Amount {
    #[serde(rename = "Ccy")]
    currency: String,
    #[serde(rename = "$value")]
    value: Decimal,
}

#[derive(Debug, Deserialize, PartialEq)]
struct AmountWithExchange {
    #[serde(rename = "Amt")]
    amount: Amount,
    #[serde(rename = "CcyXchg")]
    currency_exchange: Option<CurrencyExchange>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct CurrencyExchange {
    #[serde(rename = "SrcCcy")]
    source_currency: String,
    #[serde(rename = "TrgtCcy")]
    target_currency: String,
    // TODO: Use Enum once quick-xml serde integration fixed.
    #[serde(rename = "XchgRate")]
    exchange_rate: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct AmountDetails {
    // Actual passed amount.
    #[serde(rename = "InstdAmt")]
    instructed_amount: AmountWithExchange,
    // Specified transaction amount, before charge deduction.
    #[serde(rename = "TxAmt")]
    transaction_amount: AmountWithExchange,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Charges {
    #[serde(rename = "TtlChrgsAndTaxAmt")]
    total: Option<Amount>,
    #[serde(rename = "Rcrd")]
    records: Vec<ChargeRecord>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ChargeRecord {
    #[serde(rename = "Amt")]
    amount: Amount,
    // CRDT or DBIT
    // TODO: Use Enum once quick-xml serde integration fixed.
    #[serde(rename = "CdtDbtInd")]
    is_credit: String,
    #[serde(rename = "ChrgInclInd")]
    is_charge_included: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Date {
    #[serde(rename = "Dt")]
    date: chrono::NaiveDate,
}

fn parse_camt<R: std::io::BufRead>(reader: &mut R) -> Result<Document, DeError> {
    let document: Document = quick_xml::de::from_reader(reader)?;
    Ok(document)
}

// Debug only function.
pub fn print_camt<R: std::io::BufRead>(reader: &mut R) -> Result<String, DeError> {
    let res = parse_camt(reader)?;
    return Ok(format!("{:#?}", res));
}
