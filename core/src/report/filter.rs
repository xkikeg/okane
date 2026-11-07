//! Posting filter written in [CEL](https://cel.dev/).
//!
//! A [`PostingFilter`] is evaluated once per posting, and the posting is
//! included in the report only when the expression returns `true`.
//!
//! The expression sees these variables:
//!
//! | Variable | Type | Description |
//! |---|---|---|
//! | `date` | `timestamp` | Date of the transaction, at 00:00 UTC. |
//! | `payee` | `string` | Payee of the posting (`; Payee:` override, or the transaction payee). |
//! | `state` | `string` | `"cleared"`, `"pending"` or `"uncleared"`. Falls back to the transaction state. |
//! | `account` | `string` | Account of the posting. |
//! | `tags` | `map<string, string>` | Tags of the transaction, overlaid by the posting's own tags. |
//! | `transaction`, `t` | `map` | The enclosing transaction: `date`, `payee`, `state`, `tags` and `postings`. |
//!
//! Each element of `transaction.postings` has `account`, `payee`, `state` and
//! `tags`, the last overlaid the same way as the top-level `tags`.
//! `transaction.tags` has the transaction's tags only, including `apply tag`.
//!
//! A tag without value, such as `:trip:`, maps to `""`, so `"trip" in tags`
//! matches both `; :trip:` and `; trip: Kyoto`. Looking up a missing key is
//! an error, so use `"trip" in tags && tags["trip"] == "Kyoto"`, or the
//! optional syntax `tags[?"trip"].orValue("") == "Kyoto"`.
//!
//! Besides the CEL standard library, the strings extension (`lowerAscii()`,
//! `split()`, ...) is available.
//!
//! The `date("2024-01-31")` function builds a value comparable to `date`.

use std::{collections::HashMap, fmt, sync::Arc};

use cel::{Context, Env, ExecutionError, Program, Value, extensions};
use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::syntax::ClearState;

use super::{
    tag::TagSet,
    transaction::{Posting, Transaction},
};

/// Error on compiling or evaluating a [`PostingFilter`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FilterError {
    /// The expression is not valid CEL.
    #[error("failed to parse the filter expression:\n{0}")]
    Parse(String),
    /// The expression failed while being evaluated.
    #[error("failed to evaluate the filter expression: {0}")]
    Eval(String),
    /// The expression returned something other than a `bool`.
    #[error("filter expression must return bool, but got {0}")]
    NotBool(String),
}

/// Compiled CEL expression to select postings.
pub struct PostingFilter {
    source: String,
    program: Program,
    root: Context<'static, 'static>,
}

impl fmt::Debug for PostingFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostingFilter")
            .field("source", &self.source)
            .finish_non_exhaustive()
    }
}

impl PostingFilter {
    /// Compiles the given CEL expression.
    pub fn new(source: &str) -> Result<Self, FilterError> {
        let mut env = Env::stdlib();
        env.add_extension(extensions::strings)
            .expect("strings extension must not conflict with the standard library");
        let env = Arc::new(env);
        let program = env
            .compile(source)
            .map_err(|e| FilterError::Parse(e.to_string()))?;
        let mut root = Context::with_env(env);
        root.add_function("date", parse_date)
            .expect("date() must not conflict with the standard library");
        Ok(Self {
            source: source.to_string(),
            program,
            root,
        })
    }

    /// Returns the source expression.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns `true` if the posting at `index` of the transaction matches.
    pub(super) fn matches(&self, txn: &TxnValue, index: usize) -> Result<bool, FilterError> {
        let posting = &txn.postings[index];
        let mut ctx = self.root.new_inner_scope();
        ctx.add_variable_from_value("date", txn.date.clone());
        ctx.add_variable_from_value("payee", posting.payee.clone());
        ctx.add_variable_from_value("state", posting.state.clone());
        ctx.add_variable_from_value("account", posting.account.clone());
        ctx.add_variable_from_value("tags", posting.tags.clone());
        ctx.add_variable_from_value("transaction", txn.value.clone());
        ctx.add_variable_from_value("t", txn.value.clone());
        match self.program.execute(&ctx) {
            Ok(Value::Bool(b)) => Ok(b),
            Ok(other) => Err(FilterError::NotBool(format!("{:?}", other))),
            Err(e) => Err(FilterError::Eval(e.to_string())),
        }
    }
}

/// CEL values of a transaction, built once and shared by all its postings.
pub(super) struct TxnValue {
    date: Value,
    value: Value,
    postings: Vec<PostingValue>,
}

struct PostingValue {
    account: Value,
    payee: Value,
    state: Value,
    /// Transaction tags overlaid by the posting tags.
    tags: Value,
}

impl TxnValue {
    pub(super) fn new(txn: &Transaction<'_>) -> Self {
        let date = date_value(txn.date);
        let txn_tags = tags_value(&[txn.tags]);
        let postings: Vec<PostingValue> = txn
            .postings
            .iter()
            .map(|p| PostingValue::new(txn, &txn_tags, p))
            .collect();
        let posting_list: Vec<Value> = postings.iter().map(PostingValue::to_map).collect();
        let value = Value::from(HashMap::from([
            ("date", date.clone()),
            ("payee", Value::from(txn.payee)),
            ("state", state_value(txn.clear_state)),
            ("tags", txn_tags),
            ("postings", Value::List(Arc::new(posting_list))),
        ]));
        Self {
            date,
            value,
            postings,
        }
    }
}

impl PostingValue {
    fn new(txn: &Transaction<'_>, txn_tags: &Value, posting: &Posting<'_>) -> Self {
        // Share the transaction's map when the posting adds nothing.
        let tags = if posting.tags.is_empty() {
            txn_tags.clone()
        } else {
            tags_value(&[txn.tags, posting.tags])
        };
        Self {
            account: Value::from(posting.account.as_str()),
            payee: Value::from(posting.payee),
            state: state_value(posting.clear_state),
            tags,
        }
    }

    fn to_map(&self) -> Value {
        Value::from(HashMap::from([
            ("account", self.account.clone()),
            ("payee", self.payee.clone()),
            ("state", self.state.clone()),
            ("tags", self.tags.clone()),
        ]))
    }
}

/// Builds `map<string, string>` out of the tag sets, later ones winning.
fn tags_value(sets: &[TagSet<'_>]) -> Value {
    let mut map: HashMap<&str, Value> = HashMap::new();
    for set in sets {
        for entry in set.iter() {
            map.insert(entry.key.as_str(), Value::from(entry.value.unwrap_or("")));
        }
    }
    Value::from(map)
}

fn date_value(date: NaiveDate) -> Value {
    Value::Timestamp(midnight_utc(date))
}

fn midnight_utc(date: NaiveDate) -> DateTime<FixedOffset> {
    date.and_time(chrono::NaiveTime::MIN)
        .and_utc()
        .fixed_offset()
}

fn state_value(state: ClearState) -> Value {
    Value::from(match state {
        ClearState::Uncleared => "uncleared",
        ClearState::Cleared => "cleared",
        ClearState::Pending => "pending",
    })
}

/// `date(string)` function: parses `YYYY-MM-DD` into a timestamp at 00:00 UTC.
fn parse_date(s: Arc<String>) -> Result<DateTime<FixedOffset>, ExecutionError> {
    NaiveDate::parse_from_str(&s, "%Y-%m-%d")
        .map(midnight_utc)
        .map_err(|e| ExecutionError::function_error("date", format!("invalid date {s:?}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use std::path::PathBuf;

    use indoc::indoc;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use crate::{
        load,
        report::{ProcessOptions, ReportContext, process},
    };

    fn eval_all(input: &str, expr: &str) -> Result<Vec<(String, String)>, FilterError> {
        let path = PathBuf::from("path/to/file.ledger");
        let fake = hashmap! { path.clone() => input.as_bytes().to_vec() };
        let loader: load::Loader<load::FakeFileSystem> = load::Loader::new(path, fake.into());
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let ledger = process(&mut ctx, loader, &ProcessOptions::default()).unwrap();
        let filter = PostingFilter::new(expr)?;
        let mut got = Vec::new();
        for txn in ledger.transactions() {
            let value = TxnValue::new(txn);
            for (i, posting) in txn.postings.iter().enumerate() {
                if filter.matches(&value, i)? {
                    got.push((
                        posting.payee.to_string(),
                        posting.account.as_str().to_string(),
                    ));
                }
            }
        }
        Ok(got)
    }

    const INPUT: &str = indoc! {"
        2024/01/05 * Grocery
            Expenses:Food              10 CHF
            Assets:Bank

        2024/02/10 ! Train
            Expenses:Travel            20 CHF
            * Assets:Cash

        2024/03/15 Salary
            Assets:Bank              1000 CHF  ; Payee: Employer
            Income:Salary
    "};

    fn pair(payee: &str, account: &str) -> (String, String) {
        (payee.to_string(), account.to_string())
    }

    #[test]
    fn filter_by_date() {
        assert_eq!(
            eval_all(
                INPUT,
                r#"date >= date("2024-02-01") && date < date("2024-03-01")"#
            )
            .unwrap(),
            vec![
                pair("Train", "Expenses:Travel"),
                pair("Train", "Assets:Cash")
            ]
        );
    }

    #[test]
    fn filter_by_date_accessor() {
        assert_eq!(
            eval_all(INPUT, "date.getMonth() == 2").unwrap(),
            vec![
                pair("Employer", "Assets:Bank"),
                pair("Salary", "Income:Salary")
            ]
        );
    }

    #[test]
    fn filter_by_payee() {
        assert_eq!(
            eval_all(INPUT, r#"payee == "Employer""#).unwrap(),
            vec![pair("Employer", "Assets:Bank")]
        );
        assert_eq!(
            eval_all(INPUT, r#"t.payee == "Salary""#).unwrap(),
            vec![
                pair("Employer", "Assets:Bank"),
                pair("Salary", "Income:Salary")
            ]
        );
    }

    #[test]
    fn filter_by_state() {
        // Assets:Cash is marked cleared, overriding the pending transaction.
        assert_eq!(
            eval_all(INPUT, r#"state == "cleared""#).unwrap(),
            vec![
                pair("Grocery", "Expenses:Food"),
                pair("Grocery", "Assets:Bank"),
                pair("Train", "Assets:Cash"),
            ]
        );
        assert_eq!(
            eval_all(INPUT, r#"transaction.state == "pending""#).unwrap(),
            vec![
                pair("Train", "Expenses:Travel"),
                pair("Train", "Assets:Cash")
            ]
        );
    }

    #[test]
    fn filter_with_strings_extension() {
        assert_eq!(
            eval_all(
                INPUT,
                r#"payee.lowerAscii() == "grocery" && account == "Assets:Bank""#
            )
            .unwrap(),
            vec![pair("Grocery", "Assets:Bank")]
        );
    }

    #[test]
    fn missing_tag_is_error() {
        assert!(matches!(
            eval_all(TAGGED, r#"tags["trip"] == "Kyoto""#),
            Err(FilterError::Eval(_))
        ));
    }

    #[test]
    fn filter_by_account() {
        assert_eq!(
            eval_all(INPUT, r#"account.startsWith("Expenses:")"#).unwrap(),
            vec![
                pair("Grocery", "Expenses:Food"),
                pair("Train", "Expenses:Travel")
            ]
        );
    }

    #[test]
    fn filter_by_other_postings() {
        assert_eq!(
            eval_all(
                INPUT,
                r#"account == "Assets:Bank" && t.postings.exists(p, p.account.matches("^Expenses"))"#
            )
            .unwrap(),
            vec![pair("Grocery", "Assets:Bank")]
        );
    }

    const TAGGED: &str = indoc! {"
        apply tag trip: Kyoto

        2024/04/01 Hotel
            ; :business:
            Expenses:Travel           100 CHF  ; trip: Osaka
            Expenses:Food              20 CHF  ; :snack:
            Assets:Bank

        end apply tag

        2024/04/05 Home
            Expenses:Food              30 CHF
            Assets:Bank
    "};

    #[test]
    fn filter_by_tag_presence() {
        // `:business:` is on the transaction, so every posting sees it.
        assert_eq!(
            eval_all(TAGGED, r#""business" in tags"#).unwrap(),
            vec![
                pair("Hotel", "Expenses:Travel"),
                pair("Hotel", "Expenses:Food"),
                pair("Hotel", "Assets:Bank"),
            ]
        );
        assert_eq!(
            eval_all(TAGGED, r#""snack" in tags"#).unwrap(),
            vec![pair("Hotel", "Expenses:Food")]
        );
    }

    #[test]
    fn filter_by_tag_value_overlaid_by_posting() {
        // apply tag gives trip: Kyoto; the travel posting overrides it.
        assert_eq!(
            eval_all(TAGGED, r#"tags[?"trip"].orValue("") == "Kyoto""#).unwrap(),
            vec![pair("Hotel", "Expenses:Food"), pair("Hotel", "Assets:Bank")]
        );
        assert_eq!(
            eval_all(TAGGED, r#""trip" in tags && tags["trip"] == "Osaka""#).unwrap(),
            vec![pair("Hotel", "Expenses:Travel")]
        );
    }

    #[test]
    fn filter_by_transaction_tags() {
        // transaction.tags is not overlaid.
        assert_eq!(
            eval_all(
                TAGGED,
                r#"account == "Assets:Bank" && t.tags[?"trip"] == optional.of("Kyoto")"#
            )
            .unwrap(),
            vec![pair("Hotel", "Assets:Bank")]
        );
        assert_eq!(
            eval_all(
                TAGGED,
                r#"account == "Assets:Bank" && t.postings.exists(p, "snack" in p.tags)"#
            )
            .unwrap(),
            vec![pair("Hotel", "Assets:Bank")]
        );
    }

    #[test]
    fn invalid_syntax() {
        assert!(matches!(
            PostingFilter::new("payee =="),
            Err(FilterError::Parse(_))
        ));
    }

    #[test]
    fn non_bool_result() {
        assert!(matches!(
            eval_all(INPUT, "payee"),
            Err(FilterError::NotBool(_))
        ));
    }

    #[test]
    fn unknown_variable() {
        assert!(matches!(
            eval_all(INPUT, "amount > 0"),
            Err(FilterError::Eval(_))
        ));
    }

    #[test]
    fn invalid_date() {
        assert!(matches!(
            eval_all(INPUT, r#"date > date("2024/01/01")"#),
            Err(FilterError::Eval(_))
        ));
    }
}
