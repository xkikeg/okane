//! `okane.Amount` CEL type: an amount which may hold several commodities.
//!
//! Amounts are only partially ordered. `a < b` holds when, for every
//! commodity, `a`'s value is less than or equal to `b`'s (a missing commodity
//! counts as zero), and they differ. That is exactly when `a < b` holds
//! whatever the exchange rates are. When neither `a <= b` nor `a >= b` holds,
//! such as `100 JPY` vs `1 USD`, comparing them is an error.

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

use crate::report::{Amount, ReportContext};

static AMOUNT_TYPE: LazyLock<Type> = LazyLock::new(|| Type::new_opaque_type("okane.Amount"));

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

    /// Returns the partial order of the two.
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

impl fmt::Display for AmountVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return write!(f, "0");
        }
        for (i, (commodity, value)) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " + ")?;
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
        other.downcast_ref::<Self>().is_some_and(|o| self == o)
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
        let Some(rhs) = rhs.downcast_ref::<Self>() else {
            return Err(ExecutionError::UnexpectedType {
                got: rhs.get_type().name().to_string(),
                want: AMOUNT_TYPE.name().to_string(),
            });
        };
        self.partial_cmp(rhs).ok_or_else(|| {
            ExecutionError::function_error(
                "compare",
                format!("{self} and {rhs} cannot be ordered without exchange rates"),
            )
        })
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

/// `amount(string)`: parses `"100 JPY"`, `"JPY 100"`, `"-1,234.5 CHF"` or `"0"`.
fn amount_from_string(s: &CelString<'_>) -> Result<AmountVal, ExecutionError> {
    parse_amount(s.inner()).map_err(|msg| ExecutionError::function_error("amount", msg))
}

fn parse_amount(s: &str) -> Result<AmountVal, String> {
    let parse_value = |v: &str| v.replace(',', "").parse::<Decimal>().ok();
    let tokens: Vec<&str> = s.split_whitespace().collect();
    let (value, commodity) = match tokens.as_slice() {
        [v] => match parse_value(v) {
            Some(v) if v.is_zero() => return Ok(AmountVal::default()),
            _ => return Err(format!("{s:?} needs a commodity, such as \"100 JPY\"")),
        },
        [a, b] => match (parse_value(a), parse_value(b)) {
            (Some(v), None) => (v, *b),
            (None, Some(v)) => (v, *a),
            _ => return Err(format!("{s:?} is not an amount such as \"100 JPY\"")),
        },
        _ => return Err(format!("{s:?} is not an amount such as \"100 JPY\"")),
    };
    Ok(AmountVal::from_values(BTreeMap::from([(
        commodity.to_string(),
        value,
    )])))
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
    cel::add_overload!(env, fn amount_from_string: (CelString) -> Result<AmountVal>,
        name = "amount", id = "okane_string_to_amount")
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
    fn parse_amount_accepts_both_orders() {
        assert_eq!(Ok(amt(&[("JPY", dec!(1000))])), parse_amount("1,000 JPY"));
        assert_eq!(Ok(amt(&[("CHF", dec!(-1.5))])), parse_amount("CHF -1.5"));
        assert_eq!(Ok(AmountVal::default()), parse_amount("0"));
        assert_eq!(Ok(AmountVal::default()), parse_amount("0 JPY"));
        assert!(parse_amount("100").is_err());
        assert!(parse_amount("JPY USD").is_err());
        assert!(parse_amount("1 2 JPY").is_err());
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
