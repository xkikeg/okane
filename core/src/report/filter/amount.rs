//! `okane.Amount` CEL type: an amount which may hold several commodities.
//!
//! Amounts are only partially ordered. `a < b` holds when, for every
//! commodity, `a`'s value is less than or equal to `b`'s (a missing commodity
//! counts as zero), and they differ. That is exactly when `a < b` holds
//! whatever the exchange rates are. When neither `a <= b` nor `a >= b` holds,
//! such as `100 JPY` vs `1 USD`, the two are unordered and comparing them is
//! an error.
//!
//! `partial(amount)` gives an `okane.PartialAmount`, which tolerates that:
//! when either side of a comparison is partial, an unordered pair makes the
//! comparison *unknown* instead of failing. Unknown behaves like `NULL` in
//! SQL: `unknown || true` is `true`, `unknown && false` is `false`, and a
//! filter that ends up unknown rejects the posting.

use std::{
    any::Any,
    cmp::Ordering,
    collections::BTreeMap,
    fmt,
    sync::{Arc, LazyLock},
};

use cel::{
    Env, ExecutionError,
    common::{
        traits::{Comparer, Negator},
        types::{CelBool, CelList, CelString, Type},
        value::{StaticVal, Val},
    },
};
use rust_decimal::Decimal;

use crate::{
    report::{Amount, ReportContext},
    syntax,
};

static AMOUNT_TYPE: LazyLock<Type> = LazyLock::new(|| Type::new_opaque_type("okane.Amount"));
static PARTIAL_AMOUNT_TYPE: LazyLock<Type> =
    LazyLock::new(|| Type::new_opaque_type("okane.PartialAmount"));

/// Function name of the error raised when a comparison is unknown.
const UNKNOWN_COMPARISON: &str = "okane.partial.compare";

/// Returns `true` if the error is a comparison with `partial()` being unknown.
pub(super) fn is_unknown_comparison(err: &ExecutionError) -> bool {
    matches!(err, ExecutionError::FunctionError { function, .. } if function == UNKNOWN_COMPARISON)
}

/// CEL value of an amount: commodity name to its non-zero value.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(super) struct AmountVal(Arc<BTreeMap<String, Decimal>>);

impl AmountVal {
    pub(super) fn new(ctx: &ReportContext<'_>, amount: &Amount<'_>) -> Self {
        let values = amount
            .iter()
            .filter(|a| !a.value.is_zero())
            .map(|a| {
                (
                    a.commodity.to_str_lossy(&ctx.commodities).into_owned(),
                    a.value,
                )
            })
            .collect();
        Self(Arc::new(values))
    }

    fn from_values(values: BTreeMap<String, Decimal>) -> Self {
        Self(Arc::new(
            values.into_iter().filter(|(_, v)| !v.is_zero()).collect(),
        ))
    }

    fn map_values(&self, f: impl Fn(Decimal) -> Decimal) -> Self {
        Self::from_values(self.0.iter().map(|(k, v)| (k.clone(), f(*v))).collect())
    }
}

/// Compares commodity by commodity, a missing commodity counting as zero.
/// The two are unordered when the commodities disagree.
impl PartialOrd for AmountVal {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let mut result = Ordering::Equal;
        let keys = self.0.keys().chain(rhs.0.keys());
        for key in keys {
            let l = self.0.get(key).copied().unwrap_or_default();
            let r = rhs.0.get(key).copied().unwrap_or_default();
            match (result, l.cmp(&r)) {
                (_, Ordering::Equal) => (),
                (Ordering::Equal, ord) => result = ord,
                (prev, ord) if prev == ord => (),
                _ => return None,
            }
        }
        Some(result)
    }
}

/// Either side of an amount comparison.
struct Operand<'a> {
    amount: &'a AmountVal,
    /// Whether it's wrapped by `partial()`.
    partial: bool,
}

impl<'a> Operand<'a> {
    fn new(val: &'a dyn Val) -> Option<Self> {
        if let Some(amount) = val.downcast_ref::<AmountVal>() {
            return Some(Self {
                amount,
                partial: false,
            });
        }
        val.downcast_ref::<PartialAmountVal>().map(|p| Self {
            amount: &p.0,
            partial: true,
        })
    }

    fn compare(&self, rhs: &dyn Val) -> Result<Ordering, ExecutionError> {
        let Some(rhs) = Operand::new(rhs) else {
            return Err(ExecutionError::UnexpectedType {
                got: rhs.get_type().name().to_string(),
                want: AMOUNT_TYPE.name().to_string(),
            });
        };
        self.amount.partial_cmp(rhs.amount).ok_or_else(|| {
            let (l, r) = (self.amount, rhs.amount);
            if self.partial || rhs.partial {
                ExecutionError::function_error(
                    UNKNOWN_COMPARISON,
                    format!("{l} and {r} cannot be ordered without exchange rates"),
                )
            } else {
                ExecutionError::function_error(
                    "compare",
                    format!(
                        "{l} and {r} cannot be ordered without exchange rates; \
                         wrap either with partial() to reject such postings instead"
                    ),
                )
            }
        })
    }

    fn equals(&self, other: &dyn Val) -> bool {
        Operand::new(other).is_some_and(|o| self.amount == o.amount)
    }
}

impl fmt::Display for AmountVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("0");
        }
        for (i, (commodity, value)) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(" + ")?;
            }
            write!(f, "{} {}", value, commodity)?;
        }
        Ok(())
    }
}

impl Val for AmountVal {
    fn get_type(&self) -> &Type {
        <Self as Val>::cel_type()
    }

    fn cel_type() -> &'static Type {
        &AMOUNT_TYPE
    }

    fn as_comparer(&self) -> Option<&dyn Comparer> {
        Some(self)
    }

    fn as_negator<'b, 'v>(&'b self) -> Option<&'b (dyn Negator + 'v)>
    where
        Self: 'v,
    {
        Some(self)
    }

    fn equals(&self, other: &dyn Val) -> bool {
        Operand {
            amount: self,
            partial: false,
        }
        .equals(other)
    }

    fn clone_as_boxed<'v>(&self) -> Box<dyn Val + 'v>
    where
        Self: 'v,
    {
        Box::new(self.clone())
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl StaticVal for AmountVal {}

impl Comparer for AmountVal {
    fn compare(&self, rhs: &dyn Val) -> Result<Ordering, ExecutionError> {
        Operand {
            amount: self,
            partial: false,
        }
        .compare(rhs)
    }
}

impl Negator for AmountVal {
    fn negate<'v>(&self) -> Result<Box<dyn Val + 'v>, ExecutionError>
    where
        Self: 'v,
    {
        Ok(Box::new(self.map_values(|v| -v)))
    }
}

/// CEL value of `partial(amount)`: an amount whose comparison may be unknown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartialAmountVal(AmountVal);

impl PartialAmountVal {
    fn operand(&self) -> Operand<'_> {
        Operand {
            amount: &self.0,
            partial: true,
        }
    }
}

impl Val for PartialAmountVal {
    fn get_type(&self) -> &Type {
        <Self as Val>::cel_type()
    }

    fn cel_type() -> &'static Type {
        &PARTIAL_AMOUNT_TYPE
    }

    fn as_comparer(&self) -> Option<&dyn Comparer> {
        Some(self)
    }

    fn equals(&self, other: &dyn Val) -> bool {
        self.operand().equals(other)
    }

    fn clone_as_boxed<'v>(&self) -> Box<dyn Val + 'v>
    where
        Self: 'v,
    {
        Box::new(self.clone())
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl StaticVal for PartialAmountVal {}

impl Comparer for PartialAmountVal {
    fn compare(&self, rhs: &dyn Val) -> Result<Ordering, ExecutionError> {
        self.operand().compare(rhs)
    }
}

/// Error on parsing the argument of `amount(string)`.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
enum ParseAmountError {
    #[error("{0:?} is not an amount such as \"100 JPY\"")]
    Invalid(String),
    #[error("{0:?} needs a commodity, such as \"100 JPY\"")]
    NoCommodity(String),
}

/// `amount(string)`: parses `"100 JPY"`, `"-1,234.5 CHF"` or `"0"`.
fn amount_from_string(s: &CelString<'_>) -> Result<AmountVal, ExecutionError> {
    parse_amount(s.inner()).map_err(|e| ExecutionError::function_error("amount", e))
}

/// Parses the amount as a posting amount is, so the commodity must follow
/// the value. Only zero may go without commodity.
fn parse_amount(s: &str) -> Result<AmountVal, ParseAmountError> {
    let invalid = || ParseAmountError::Invalid(s.to_string());
    let trimmed = s.trim();
    // TODO: Drop this once the parser no longer hangs on an empty input.
    if trimmed.is_empty() {
        return Err(invalid());
    }
    let Ok(syntax::expr::ValueExpr::Amount(amount)) = syntax::expr::ValueExpr::try_from(trimmed)
    else {
        return Err(invalid());
    };
    let value = Decimal::from(amount.value);
    if value.is_zero() {
        return Ok(AmountVal::default());
    }
    if amount.commodity.is_empty() {
        return Err(ParseAmountError::NoCommodity(s.to_string()));
    }
    Ok(AmountVal::from_values(BTreeMap::from([(
        amount.commodity.into_owned(),
        value,
    )])))
}

fn amount_partial(this: &AmountVal) -> PartialAmountVal {
    PartialAmountVal(this.clone())
}

fn amount_is_zero(this: &AmountVal) -> CelBool {
    CelBool::from(this.0.is_empty())
}

fn amount_abs(this: &AmountVal) -> AmountVal {
    this.map_values(|v| v.abs())
}

fn amount_commodities(this: &AmountVal) -> CelList<'static> {
    let names: Vec<Box<dyn Val>> = this
        .0
        .keys()
        .map(|k| Box::new(CelString::from(k.clone())) as Box<dyn Val>)
        .collect();
    CelList::from(names)
}

/// Registers `okane.Amount` and its functions on the `env`.
pub(super) fn register(env: &mut Env) {
    env.add_type(AMOUNT_TYPE.to_owned())
        .expect("okane.Amount must be unique");
    env.add_type(PARTIAL_AMOUNT_TYPE.to_owned())
        .expect("okane.PartialAmount must be unique");
    cel::add_overload!(env, fn amount_from_string: (CelString) -> Result<AmountVal>,
        name = "amount", id = "okane_string_to_amount")
    .expect("must be unique");
    cel::add_overload!(env, fn amount_partial: (AmountVal) -> PartialAmountVal,
        name = "partial", id = "okane_amount_to_partial")
    .expect("must be unique");
    cel::add_member_overload!(env, fn amount_is_zero: (AmountVal) -> CelBool,
        name = "isZero", id = "okane_amount_is_zero")
    .expect("must be unique");
    cel::add_member_overload!(env, fn amount_abs: (AmountVal) -> AmountVal,
        name = "abs", id = "okane_amount_abs")
    .expect("must be unique");
    cel::add_member_overload!(env, fn amount_commodities: (AmountVal) -> CelList,
        name = "commodities", id = "okane_amount_commodities")
    .expect("must be unique");
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    fn amt(values: &[(&str, Decimal)]) -> AmountVal {
        AmountVal::from_values(values.iter().map(|(k, v)| (k.to_string(), *v)).collect())
    }

    #[test]
    fn parse_amount_takes_ledger_syntax() {
        assert_eq!(Ok(amt(&[("JPY", dec!(1000))])), parse_amount("1,000 JPY"));
        assert_eq!(Ok(amt(&[("CHF", dec!(-1.5))])), parse_amount(" -1.5CHF "));
        assert_eq!(Ok(AmountVal::default()), parse_amount("0"));
        assert_eq!(Ok(AmountVal::default()), parse_amount("0 JPY"));
        assert_eq!(
            Err(ParseAmountError::NoCommodity("100".to_string())),
            parse_amount("100")
        );
        // A commodity before the value isn't supported.
        for invalid in ["JPY 100", "JPY USD", "1 2 JPY", ""] {
            assert_eq!(
                Err(ParseAmountError::Invalid(invalid.to_string())),
                parse_amount(invalid)
            );
        }
    }

    #[test]
    fn partial_order() {
        let jpy100 = amt(&[("JPY", dec!(100))]);
        let jpy200 = amt(&[("JPY", dec!(200))]);
        let usd1 = amt(&[("USD", dec!(1))]);
        let both = amt(&[("JPY", dec!(200)), ("USD", dec!(1))]);
        let zero = AmountVal::default();
        assert_eq!(Some(Ordering::Less), jpy100.partial_cmp(&jpy200));
        assert_eq!(Some(Ordering::Equal), jpy100.partial_cmp(&jpy100));
        assert_eq!(None, jpy100.partial_cmp(&usd1));
        assert_eq!(Some(Ordering::Less), jpy100.partial_cmp(&both));
        assert_eq!(Some(Ordering::Greater), usd1.partial_cmp(&zero));
        assert_eq!(
            Some(Ordering::Less),
            amt(&[("JPY", dec!(-100))]).partial_cmp(&usd1)
        );
    }

    #[test]
    fn display() {
        assert_eq!("0", AmountVal::default().to_string());
        assert_eq!(
            "200 JPY + 1 USD",
            amt(&[("USD", dec!(1)), ("JPY", dec!(200))]).to_string()
        );
    }
}
