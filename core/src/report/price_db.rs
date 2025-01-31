//! Provides [PriceRepository], which can compute the commodity (currency) conversion.

use std::{
    collections::{hash_map, BinaryHeap, HashMap},
    path::Path,
};

use chrono::{NaiveDate, TimeDelta};
use rust_decimal::Decimal;

use crate::parse;

use super::{
    commodity::Commodity,
    context::ReportContext,
    eval::{Amount, SingleAmount},
};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse price DB entry: {0}")]
    Parse(#[from] parse::ParseError),
}

/// Source of the price information.
/// In the DB, latter one (larger one as Ord) has priority,
/// and if you have events with higher priority,
/// lower priority events are discarded.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(super) enum PriceSource {
    Ledger,
    PriceDB,
}

#[derive(Debug)]
struct Entry(PriceSource, Vec<(NaiveDate, Decimal)>);

/// Builder of [`PriceRepository`].
#[derive(Debug, Default)]
pub(super) struct PriceRepositoryBuilder<'ctx> {
    records: HashMap<Commodity<'ctx>, HashMap<Commodity<'ctx>, Entry>>,
}

/// Event of commodity price.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct PriceEvent<'ctx> {
    pub date: NaiveDate,
    pub price_x: SingleAmount<'ctx>,
    pub price_y: SingleAmount<'ctx>,
}

#[cfg(test)]
impl<'ctx> PriceEvent<'ctx> {
    fn sort_key<'a>(&'a self) -> (NaiveDate, &'ctx str, &'ctx str, &'a Decimal, &'a Decimal) {
        let PriceEvent {
            date,
            price_x:
                SingleAmount {
                    value: value_x,
                    commodity: commodity_x,
                },
            price_y:
                SingleAmount {
                    value: value_y,
                    commodity: commodity_y,
                },
        } = self;
        (
            *date,
            commodity_x.as_str(),
            commodity_y.as_str(),
            value_x,
            value_y,
        )
    }
}

impl<'ctx> PriceRepositoryBuilder<'ctx> {
    pub fn insert_price(&mut self, source: PriceSource, event: PriceEvent<'ctx>) {
        if event.price_x.commodity == event.price_y.commodity {
            // this must be an error returned, instead of log error.
            log::error!("price log should not contain the self-mention rate");
        }
        self.insert_impl(source, event.date, event.price_x, event.price_y);
        self.insert_impl(source, event.date, event.price_y, event.price_x);
    }

    fn insert_impl(
        &mut self,
        source: PriceSource,
        date: NaiveDate,
        price_of: SingleAmount<'ctx>,
        price_with: SingleAmount<'ctx>,
    ) {
        let Entry(stored_source, entries): &mut _ = self
            .records
            .entry(price_with.commodity)
            .or_default()
            .entry(price_of.commodity)
            .or_insert(Entry(PriceSource::Ledger, Vec::new()));
        if *stored_source < source {
            *stored_source = source;
            entries.clear();
        }
        // price_of: x X
        // price_with: y Y
        //
        // typical use: price_of: 1 X
        // then records[Y][X] == y (/ 1)
        entries.push((date, price_with.value / price_of.value));
    }

    /// Loads PriceDB information from the given file.
    pub fn load_price_db(
        &mut self,
        ctx: &mut ReportContext<'ctx>,
        path: &Path,
    ) -> Result<(), LoadError> {
        // Even though price db can be up to a few megabytes,
        // still it's much easier to load everything into memory.
        let content = std::fs::read_to_string(path)?;
        for entry in parse::price::parse_price_db(&parse::ParseOptions::default(), &content) {
            let (_, entry) = entry?;
            // we cannot skip commodities we don't know, as the price might be indirected in DB.
            // For example, if we have only AUD and JPY in Ledger,
            // price DB might just expose AUD/EUR EUR/CHF CHF/JPY.
            let target = ctx.commodities.ensure(entry.target.as_ref());
            let rate: SingleAmount<'ctx> = SingleAmount::from_value(
                entry.rate.value.value,
                ctx.commodities.ensure(&entry.rate.commodity),
            );
            self.insert_price(
                PriceSource::PriceDB,
                PriceEvent {
                    price_x: SingleAmount::from_value(Decimal::ONE, target),
                    price_y: rate,
                    date: entry.datetime.date(),
                },
            );
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn into_events(self) -> Vec<PriceEvent<'ctx>> {
        let mut ret = Vec::new();
        for (price_with, v) in self.records {
            for (price_of, Entry(_, v)) in v {
                for (date, rate) in v {
                    ret.push(PriceEvent {
                        price_x: SingleAmount::from_value(Decimal::ONE, price_of),
                        price_y: SingleAmount::from_value(rate, price_with),
                        date,
                    });
                }
            }
        }
        ret.sort_by(|x, y| x.sort_key().cmp(&y.sort_key()));
        ret
    }

    pub fn build(self) -> PriceRepository<'ctx> {
        PriceRepository::new(self.build_naive())
    }

    fn build_naive(mut self) -> NaivePriceRepository<'ctx> {
        self.records
            .values_mut()
            .for_each(|x| x.values_mut().for_each(|x| x.1.sort()));
        NaivePriceRepository {
            records: self.records,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConversionError<'ctx> {
    #[error("commodity rate {0} into {1} at {2} not found")]
    RateNotFound(SingleAmount<'ctx>, Commodity<'ctx>, NaiveDate),
}

/// Converts the given amount into the specified commodity.
pub fn convert_amount<'ctx>(
    price_repos: &mut PriceRepository<'ctx>,
    amount: &Amount<'ctx>,
    commodity_with: Commodity<'ctx>,
    date: NaiveDate,
) -> Result<Amount<'ctx>, ConversionError<'ctx>> {
    let mut result = Amount::zero();
    for v in amount.iter() {
        result += price_repos.convert_single(v, commodity_with, date)?;
    }
    Ok(result)
}

/// Repository which user can query the conversion rate with.
pub struct PriceRepository<'ctx> {
    inner: NaivePriceRepository<'ctx>,
    // TODO: add price_with as a key, otherwise it's wrong.
    // BTreeMap could be used if cursor support is ready.
    // Then, we can avoid computing rates over and over if no rate update happens.
    cache: HashMap<(Commodity<'ctx>, NaiveDate), HashMap<Commodity<'ctx>, WithDistance<Decimal>>>,
}

impl<'ctx> PriceRepository<'ctx> {
    fn new(inner: NaivePriceRepository<'ctx>) -> Self {
        Self {
            inner,
            cache: HashMap::new(),
        }
    }

    /// Converts the given `value` into the `commodity_with`.
    /// If the given value has already the `commodity_with`,
    /// returns `Ok(value)` as-is.
    pub fn convert_single(
        &mut self,
        value: SingleAmount<'ctx>,
        commodity_with: Commodity<'ctx>,
        date: NaiveDate,
    ) -> Result<SingleAmount<'ctx>, ConversionError<'ctx>> {
        if value.commodity == commodity_with {
            return Ok(value);
        }
        let rate = self
            .cache
            .entry((commodity_with, date))
            .or_insert_with(|| self.inner.compute_price_table(commodity_with, date))
            .get(&value.commodity);
        match rate {
            Some(WithDistance(_, rate)) => {
                Ok(SingleAmount::from_value(value.value * rate, commodity_with))
            }
            None => Err(ConversionError::RateNotFound(value, commodity_with, date)),
        }
    }
}

#[derive(Debug)]
struct NaivePriceRepository<'ctx> {
    // from comodity -> to commodity -> date -> price.
    // e.g. USD AAPL 2024-01-01 100 means 1 AAPL == 100 USD at 2024-01-01.
    // the value are sorted in NaiveDate order.
    records: HashMap<Commodity<'ctx>, HashMap<Commodity<'ctx>, Entry>>,
}

impl<'ctx> NaivePriceRepository<'ctx> {
    /// Copied from CachedPriceRepository, needs to be factored out properly.
    #[cfg(test)]
    fn convert(
        &self,
        value: SingleAmount<'ctx>,
        commodity_with: Commodity<'ctx>,
        date: NaiveDate,
    ) -> Result<SingleAmount<'ctx>, SingleAmount<'ctx>> {
        if value.commodity == commodity_with {
            return Ok(value);
        }
        let rate = self
            .compute_price_table(commodity_with, date)
            .get(&value.commodity)
            .map(|x| x.1);
        match rate {
            Some(rate) => Ok(SingleAmount::from_value(value.value * rate, commodity_with)),
            None => Err(value),
        }
    }

    fn compute_price_table(
        &self,
        price_with: Commodity<'ctx>,
        date: NaiveDate,
    ) -> HashMap<Commodity<'ctx>, WithDistance<Decimal>> {
        // minimize the distance, and then minimize the staleness.
        let mut queue: BinaryHeap<WithDistance<(Commodity<'ctx>, Decimal)>> = BinaryHeap::new();
        let mut distances: HashMap<Commodity<'ctx>, WithDistance<Decimal>> = HashMap::new();
        queue.push(WithDistance(
            Distance {
                num_ledger_conversions: 0,
                num_all_conversions: 0,
                staleness: TimeDelta::zero(),
            },
            (price_with, Decimal::ONE),
        ));
        while let Some(curr) = queue.pop() {
            log::debug!("curr: {:?}", curr);
            let WithDistance(curr_dist, (prev, prev_rate)) = curr;
            if let Some(WithDistance(prev_dist, _)) = distances.get(&prev) {
                if *prev_dist < curr_dist {
                    log::debug!(
                        "no need to update, prev_dist {:?} is smaller than curr_dist {:?}",
                        prev_dist,
                        curr_dist
                    );
                    continue;
                }
            }
            for (j, Entry(source, rates)) in match self.records.get(&prev) {
                None => continue,
                Some(x) => x,
            } {
                let bound = rates.partition_point(|(record_date, _)| record_date <= &date);
                log::debug!(
                    "found next commodity {} with date bound {}",
                    j.as_str(),
                    bound
                );
                if bound == 0 {
                    // we cannot find any rate information at the date (all rates are in future).
                    // let's treat rates are not available.
                    continue;
                }
                let (record_date, rate) = rates[bound - 1];
                let next_dist = curr_dist.extend(*source, date - record_date);
                let rate = prev_rate * rate;
                let next = WithDistance(next_dist.clone(), (*j, rate));
                let updated = match distances.entry(*j) {
                    hash_map::Entry::Occupied(mut e) => {
                        if e.get() <= &next_dist {
                            false
                        } else {
                            e.insert(WithDistance(next_dist, rate));
                            true
                        }
                    }
                    hash_map::Entry::Vacant(e) => {
                        e.insert(WithDistance(next_dist, rate));
                        true
                    }
                };
                if !updated {
                    continue;
                }
                queue.push(next);
            }
        }
        distances
    }
}

/// Distance to minimize during the price DB computation.
///
/// Now this is using simple derived [Ord] logic,
/// but we can work on heuristic cost function instead.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Distance {
    /// Number of conversions with [`PriceSource::Ledger`] used to compute the rate.
    /// Minimize this because we assume [`PriceSource::PriceDB`] is more reliable
    /// than the one in Ledger.
    num_ledger_conversions: usize,
    /// Number of conversions used to compute the rate.
    num_all_conversions: usize,
    /// Staleness of the conversion rate.
    staleness: TimeDelta,
}

impl Distance {
    fn extend(&self, source: PriceSource, staleness: TimeDelta) -> Self {
        let num_ledger_conversions = self.num_ledger_conversions
            + match source {
                PriceSource::Ledger => 1,
                PriceSource::PriceDB => 0,
            };
        Self {
            num_ledger_conversions,
            num_all_conversions: self.num_all_conversions + 1,
            staleness: std::cmp::max(self.staleness, staleness),
        }
    }
}

#[derive(Debug)]
struct WithDistance<T>(Distance, T);

impl<T> PartialEq for WithDistance<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> PartialEq<Distance> for WithDistance<T> {
    fn eq(&self, other: &Distance) -> bool {
        self.0 == *other
    }
}

impl<T> Eq for WithDistance<T> {}

impl<T> PartialOrd for WithDistance<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Eq> PartialOrd<Distance> for WithDistance<T> {
    fn partial_cmp(&self, other: &Distance) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T: Eq> Ord for WithDistance<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bumpalo::Bump;
    use rust_decimal_macros::dec;

    #[test]
    fn price_db_computes_direct_price() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let chf = ctx.commodities.ensure("CHF");
        let eur = ctx.commodities.ensure("EUR");
        let mut builder = PriceRepositoryBuilder::default();
        builder.insert_price(
            PriceSource::Ledger,
            PriceEvent {
                date: NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
                price_x: SingleAmount::from_value(dec!(1), eur),
                price_y: SingleAmount::from_value(dec!(0.8), chf),
            },
        );

        let db = builder.build_naive();

        // before the event date, we can't convert the value, thus see Right.
        let got = db.convert(
            SingleAmount::from_value(dec!(1), eur),
            chf,
            NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
        );
        assert_eq!(got, Err(SingleAmount::from_value(dec!(1), eur)));

        let got = db.convert(
            SingleAmount::from_value(dec!(10), chf),
            eur,
            NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
        );
        assert_eq!(got, Err(SingleAmount::from_value(dec!(10), chf)));

        let got = db.convert(
            SingleAmount::from_value(dec!(1.0), eur),
            chf,
            NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
        );
        assert_eq!(got, Ok(SingleAmount::from_value(dec!(0.8), chf)));

        let got = db.convert(
            SingleAmount::from_value(dec!(10.0), chf),
            eur,
            NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
        );
        assert_eq!(got, Ok(SingleAmount::from_value(dec!(12.5), eur)));
    }

    #[test]
    fn price_db_computes_indirect_price() {
        let arena = Bump::new();
        let mut ctx = ReportContext::new(&arena);
        let chf = ctx.commodities.ensure("CHF");
        let eur = ctx.commodities.ensure("EUR");
        let usd = ctx.commodities.ensure("USD");
        let jpy = ctx.commodities.ensure("JPY");
        let mut builder = PriceRepositoryBuilder::default();

        builder.insert_price(
            PriceSource::Ledger,
            PriceEvent {
                date: NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
                price_x: SingleAmount::from_value(dec!(0.8), chf),
                price_y: SingleAmount::from_value(dec!(1), eur),
            },
        );
        builder.insert_price(
            PriceSource::Ledger,
            PriceEvent {
                date: NaiveDate::from_ymd_opt(2024, 10, 2).unwrap(),
                price_x: SingleAmount::from_value(dec!(0.8), eur),
                price_y: SingleAmount::from_value(dec!(1), usd),
            },
        );
        builder.insert_price(
            PriceSource::Ledger,
            PriceEvent {
                date: NaiveDate::from_ymd_opt(2024, 10, 3).unwrap(),
                price_x: SingleAmount::from_value(dec!(100), jpy),
                price_y: SingleAmount::from_value(dec!(1), usd),
            },
        );

        // 1 EUR = 0.8 CHF
        // 1 USD = 0.8 EUR
        // 1 USD = 100 JPY
        // 1 CHF == 5/4 EUR == (5/4)*(5/4) USD == 156.25 JPY

        let db = builder.build_naive();

        let got = db.convert(
            SingleAmount::from_value(dec!(1), chf),
            jpy,
            NaiveDate::from_ymd_opt(2024, 10, 3).unwrap(),
        );
        assert_eq!(got, Ok(SingleAmount::from_value(dec!(156.25), jpy)));
    }
}
