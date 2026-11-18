//! `convert(amount, commodity)` and `convert(amount, commodity, date)`:
//! converts an amount into the commodity, with the rate as of the date.
//! Without `date`, the rate is as of the date of the transaction.
//!
//! The CEL function can't reach the price database, as it has to be
//! `'static`. Instead it finds the rates in a [`Rates`] bound to a hidden
//! variable. When a rate is missing there, it leaves a request and fails.
//! [`PostingFilter::matches`](super::PostingFilter::matches) then asks
//! [`Rates::resolve_pending`] to answer the request out of the price database,
//! and evaluates the expression again.

use std::{
    any::Any,
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock, Mutex, MutexGuard},
};

use cel::{
    Context, ExecutionError, FunctionContext,
    common::{
        types::{CelString, CelTimestamp, Type},
        value::{CowVal, StaticVal, Val},
    },
};
use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::report::{ReportContext, SingleAmount, price_db::PriceRepository};

use super::amount::AmountVal;

/// Name of the variable holding [`Rates`]. It isn't a valid identifier, so
/// the expression can't refer to it.
pub(super) const RATES_VARIABLE: &str = "@okane.rates";

const FUNCTION: &str = "convert";

static RATES_TYPE: LazyLock<Type> = LazyLock::new(|| Type::new_opaque_type("okane.Rates"));

/// Rate to convert one unit of `from` into `to` as of `date`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RateKey {
    from: String,
    to: String,
    date: NaiveDate,
}

#[derive(Debug, Default)]
struct RateTable {
    /// Rates asked so far, or the reason why it's not available.
    known: HashMap<RateKey, Result<Decimal, String>>,
    /// Rate `convert()` needed and didn't find in `known`.
    pending: Option<RateKey>,
}

/// Rates `convert()` uses, shared by the postings of a transaction.
#[derive(Debug, Clone)]
pub(super) struct Rates {
    /// Date of the transaction, used when `convert()` is given no date.
    date: NaiveDate,
    table: Arc<Mutex<RateTable>>,
}

impl Rates {
    pub(super) fn new(date: NaiveDate) -> Self {
        Self {
            date,
            table: Arc::default(),
        }
    }

    fn table(&self) -> MutexGuard<'_, RateTable> {
        // The table stays consistent even if a holder of the lock panicked.
        self.table
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Looks the rate the last evaluation missed up in the price database.
    /// Returns `false` if the evaluation missed nothing.
    pub(super) fn resolve_pending<'ctx>(
        &self,
        ctx: &ReportContext<'ctx>,
        price_repos: &mut PriceRepository<'ctx>,
    ) -> bool {
        let mut table = self.table();
        let Some(key) = table.pending.take() else {
            return false;
        };
        let rate = lookup_rate(ctx, price_repos, &key);
        table.known.insert(key, rate);
        true
    }

    fn convert(
        &self,
        amount: &AmountVal,
        to: &str,
        date: NaiveDate,
    ) -> Result<AmountVal, ExecutionError> {
        let mut table = self.table();
        let mut total = Decimal::ZERO;
        for (commodity, value) in amount.iter() {
            if commodity == to {
                total += value;
                continue;
            }
            let key = RateKey {
                from: commodity.to_string(),
                to: to.to_string(),
                date,
            };
            match table.known.get(&key) {
                Some(Ok(rate)) => total += value * rate,
                Some(Err(msg)) => return Err(ExecutionError::function_error(FUNCTION, msg)),
                None => {
                    let err = ExecutionError::function_error(
                        FUNCTION,
                        format!(
                            "rate of {} into {} at {} is not loaded",
                            key.from, key.to, date
                        ),
                    );
                    table.pending = Some(key);
                    return Err(err);
                }
            }
        }
        Ok(AmountVal::from_values(BTreeMap::from([(
            to.to_string(),
            total,
        )])))
    }
}

fn lookup_rate<'ctx>(
    ctx: &ReportContext<'ctx>,
    price_repos: &mut PriceRepository<'ctx>,
    key: &RateKey,
) -> Result<Decimal, String> {
    let resolve = |name: &str| {
        ctx.commodities
            .resolve(name)
            .ok_or_else(|| format!("commodity {name} not found"))
    };
    let from = resolve(&key.from)?;
    let to = resolve(&key.to)?;
    price_repos
        .convert_single(
            ctx,
            SingleAmount::from_value(from, Decimal::ONE),
            to,
            key.date,
        )
        .map(|converted| converted.value)
        .map_err(|e| e.to_string())
}

impl Val for Rates {
    fn get_type(&self) -> &Type {
        <Self as Val>::cel_type()
    }

    fn cel_type() -> &'static Type {
        &RATES_TYPE
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

impl StaticVal for Rates {}

fn unexpected_type(got: &dyn Val, want: &Type) -> ExecutionError {
    ExecutionError::UnexpectedType {
        got: got.get_type().name().to_string(),
        want: want.name().to_string(),
    }
}

fn convert<'context, 'call>(
    ftx: &mut FunctionContext<'context, 'call>,
) -> Result<CowVal<'context, 'call>, ExecutionError> {
    let (amount, commodity, date) = match ftx.args.as_slice() {
        [amount, commodity] => (amount, commodity, None),
        [amount, commodity, date] => (amount, commodity, Some(date)),
        args => return Err(ExecutionError::invalid_argument_count(3, args.len())),
    };
    let amount = amount
        .downcast_ref::<AmountVal>()
        .ok_or_else(|| unexpected_type(amount.as_ref(), AmountVal::cel_type()))?;
    let commodity = commodity
        .downcast_ref::<CelString>()
        .ok_or_else(|| unexpected_type(commodity.as_ref(), CelString::cel_type()))?;
    let date = date
        .map(|date| {
            date.downcast_ref::<CelTimestamp>()
                .map(|t| t.inner().date_naive())
                .ok_or_else(|| unexpected_type(date.as_ref(), CelTimestamp::cel_type()))
        })
        .transpose()?;
    let rates = ftx
        .ptx
        .get_variable(RATES_VARIABLE)
        .ok_or_else(|| ftx.error("rates are not available"))?;
    let rates = rates
        .downcast_ref::<Rates>()
        .ok_or_else(|| ftx.error("rates are not available"))?;
    let converted = rates.convert(amount, commodity.inner(), date.unwrap_or(rates.date))?;
    Ok(CowVal::owned(converted))
}

/// Function taking the arguments as they are. `convert()` needs that to
/// reach the variable, and to take two or three arguments.
type RawFunction = Box<
    dyn for<'context, 'call> Fn(
            &mut FunctionContext<'context, 'call>,
        ) -> Result<CowVal<'context, 'call>, ExecutionError>
        + Send
        + Sync,
>;

/// Registers `convert()` on the root context.
pub(super) fn register(root: &mut Context<'static, 'static>) {
    let function: RawFunction = Box::new(convert);
    root.add_function(FUNCTION, function)
        .expect("convert() must not conflict with the standard library");
}
