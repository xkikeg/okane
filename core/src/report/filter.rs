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
//! | `amount` | `okane.Amount` | Amount of the posting, as written (not converted). |
//! | `tags` | `map<string, string>` | Tags of the transaction, overlaid by the posting's own tags. |
//! | `transaction`, `t` | `map` | The enclosing transaction: `date`, `payee`, `state`, `tags` and `postings`. |
//!
//! Each element of `transaction.postings` has `account`, `payee`, `state`,
//! `amount` and `tags`, the last overlaid the same way as the top-level `tags`.
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
//!
//! `amount("100 JPY")` builds an amount. Amounts support `==`, `!=`, unary
//! `-`, and `<`, `<=`, `>`, `>=` as a partial order: comparing amounts whose
//! order depends on exchange rates, such as `100 JPY` and `1 USD`, is an
//! error. `amount("0")` has no commodity, so it compares with any amount.
//! Methods: `isZero()`, `abs()` and `commodities()` (sorted `list<string>`).

mod amount;

use std::{collections::HashMap, fmt, sync::Arc};

use cel::{
    Context, Env, ExecutionError, Program, Value,
    common::{
        types::{CelList, CelMap, CelMapKey as Key},
        value::Val,
    },
    extensions,
};
use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::syntax::ClearState;

use super::{
    context::ReportContext,
    tag::TagSet,
    transaction::{Posting, Transaction},
};
use amount::AmountVal;

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
        amount::register(&mut env);
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
        ctx.add_variable_as_val("date", txn.date.clone_as_boxed());
        ctx.add_variable_as_val("payee", posting.payee.clone_as_boxed());
        ctx.add_variable_as_val("state", posting.state.clone_as_boxed());
        ctx.add_variable_as_val("account", posting.account.clone_as_boxed());
        ctx.add_variable_as_val("amount", posting.amount.clone_as_boxed());
        ctx.add_variable_as_val("tags", posting.tags.clone_as_boxed());
        ctx.add_variable_as_val("transaction", txn.value.clone_as_boxed());
        ctx.add_variable_as_val("t", txn.value.clone_as_boxed());
        match self.program.execute(&ctx) {
            Ok(Value::Bool(b)) => Ok(b),
            Ok(other) => Err(FilterError::NotBool(format!("{:?}", other))),
            Err(e) => Err(FilterError::Eval(e.to_string())),
        }
    }
}

type BoxVal = Box<dyn Val + 'static>;

/// CEL values of a transaction, built once and shared by all its postings.
pub(super) struct TxnValue {
    date: BoxVal,
    value: BoxVal,
    postings: Vec<PostingValue>,
}

struct PostingValue {
    account: BoxVal,
    payee: BoxVal,
    state: BoxVal,
    amount: BoxVal,
    /// Transaction tags overlaid by the posting tags.
    tags: BoxVal,
}

impl TxnValue {
    pub(super) fn new(ctx: &ReportContext<'_>, txn: &Transaction<'_>) -> Self {
        let date = val(Value::Timestamp(midnight_utc(txn.date)));
        let txn_tags = tags_value(&[txn.tags]);
        let postings: Vec<PostingValue> = txn
            .postings
            .iter()
            .map(|p| PostingValue::new(ctx, txn, &txn_tags, p))
            .collect();
        let posting_list: Vec<BoxVal> = postings.iter().map(PostingValue::to_map).collect();
        let value = map_val([
            ("date", date.clone_as_boxed()),
            ("payee", val(txn.payee)),
            ("state", state_value(txn.clear_state)),
            ("tags", txn_tags),
            ("postings", Box::new(CelList::from(posting_list))),
        ]);
        Self {
            date,
            value,
            postings,
        }
    }
}

impl PostingValue {
    fn new(
        ctx: &ReportContext<'_>,
        txn: &Transaction<'_>,
        txn_tags: &BoxVal,
        posting: &Posting<'_>,
    ) -> Self {
        let tags = if posting.tags.is_empty() {
            txn_tags.clone_as_boxed()
        } else {
            tags_value(&[txn.tags, posting.tags])
        };
        Self {
            account: val(posting.account.as_str()),
            payee: val(posting.payee),
            state: state_value(posting.clear_state),
            amount: Box::new(AmountVal::new(ctx, &posting.amount)),
            tags,
        }
    }

    fn to_map(&self) -> BoxVal {
        map_val([
            ("account", self.account.clone_as_boxed()),
            ("payee", self.payee.clone_as_boxed()),
            ("state", self.state.clone_as_boxed()),
            ("amount", self.amount.clone_as_boxed()),
            ("tags", self.tags.clone_as_boxed()),
        ])
    }
}

/// Converts a plain CEL [`Value`] into a [`Val`].
fn val(v: impl Into<Value>) -> BoxVal {
    v.into()
        .try_into()
        .expect("plain values must convert into Val")
}

fn map_val<const N: usize>(entries: [(&'static str, BoxVal); N]) -> BoxVal {
    let map: HashMap<Key<'static>, BoxVal> = entries
        .into_iter()
        .map(|(k, v)| (Key::from(k), v))
        .collect();
    Box::new(CelMap::from(map))
}

/// Builds `map<string, string>` out of the tag sets, later ones winning.
fn tags_value(sets: &[TagSet<'_>]) -> BoxVal {
    let mut map: HashMap<&str, Value> = HashMap::new();
    for set in sets {
        for entry in set.iter() {
            map.insert(entry.key.as_str(), Value::from(entry.value.unwrap_or("")));
        }
    }
    val(map)
}

fn midnight_utc(date: NaiveDate) -> DateTime<FixedOffset> {
    date.and_time(chrono::NaiveTime::MIN)
        .and_utc()
        .fixed_offset()
}

fn state_value(state: ClearState) -> BoxVal {
    val(match state {
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
            let value = TxnValue::new(&ctx, txn);
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

    const MULTI: &str = indoc! {"
        2024/05/01 Exchange
            Assets:JPY              10,000 JPY
            Assets:USD                 -70 USD

        2024/05/02 Coffee
            Expenses:Food              500 JPY
            Assets:JPY

        2024/05/03 Book
            Expenses:Book               20 USD
            Assets:USD
    "};

    #[test]
    fn filter_by_amount_same_commodity() {
        assert_eq!(
            eval_all(
                MULTI,
                r#""JPY" in amount.commodities() && amount >= amount("1,000 JPY")"#
            )
            .unwrap(),
            vec![pair("Exchange", "Assets:JPY")]
        );
        assert_eq!(
            // Equality never fails, even between different commodities.
            eval_all(MULTI, r#"amount == amount("JPY -500")"#).unwrap(),
            vec![pair("Coffee", "Assets:JPY")]
        );
    }

    #[test]
    fn filter_by_amount_against_zero() {
        // Zero has no commodity, so it is comparable with every amount.
        assert_eq!(
            eval_all(
                MULTI,
                r#"account.startsWith("Assets:") && amount < amount("0")"#
            )
            .unwrap(),
            vec![
                pair("Exchange", "Assets:USD"),
                pair("Coffee", "Assets:JPY"),
                pair("Book", "Assets:USD"),
            ]
        );
    }

    #[test]
    fn filter_by_amount_abs_and_negate() {
        assert_eq!(
            eval_all(MULTI, r#"-amount == amount("70 USD")"#).unwrap(),
            vec![pair("Exchange", "Assets:USD")]
        );
        assert_eq!(
            eval_all(
                MULTI,
                r#""USD" in amount.commodities() && amount.abs() >= amount("50 USD")"#
            )
            .unwrap(),
            vec![pair("Exchange", "Assets:USD")]
        );
    }

    #[test]
    fn filter_by_other_posting_amount() {
        assert_eq!(
            eval_all(
                MULTI,
                r#"account == "Assets:USD" && t.postings.exists(p, p.account.startsWith("Expenses:") && p.amount > amount("10 USD"))"#
            )
            .unwrap(),
            vec![pair("Book", "Assets:USD")]
        );
    }

    #[test]
    fn amount_different_commodities_not_comparable() {
        let err = eval_all(MULTI, r#"amount > amount("1 USD")"#).unwrap_err();
        assert!(
            matches!(&err, FilterError::Eval(msg) if msg.contains("without exchange rates")),
            "{err:?}"
        );
    }

    #[test]
    fn amount_invalid_literal() {
        assert!(matches!(
            eval_all(MULTI, r#"amount > amount("100")"#),
            Err(FilterError::Eval(_))
        ));
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
