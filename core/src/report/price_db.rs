//! Provides [PriceRepository], which can compute the commodity (currency) conversion.

use std::{
    collections::{hash_map, BinaryHeap, HashMap},
    path::Path,
};

use chrono::{NaiveDate, TimeDelta};
use rust_decimal::Decimal;
use winnow::Parser as _;

use super::commodity::Commodity;

#[derive(Debug, thiserror::Error)]
pub enum PriceDBError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse price DB entry: {0}")]
    Parse(String),
}

/// Loads PriceDB information from the given file.
pub fn load_price_db(path: &Path) -> Result<(), PriceDBError> {
    // Even though price db can be up to a few megabytes,
    // still it's much easier to load everything into memory.
    let before = chrono::Local::now();
    let content = std::fs::read_to_string(path)?;
    let input: &str = &content;
    let result: Vec<_> = winnow::combinator::preceded(
        winnow::ascii::space0,
        winnow::combinator::repeat(
            0..,
            winnow::combinator::terminated(
                crate::parse::price::price_db_entry,
                winnow::ascii::space0,
            ),
        ),
    )
    .parse(input)
    .map_err(|x| PriceDBError::Parse(format!("{}", x)))?;
    let after = chrono::Local::now();
    log::info!("TODO: use this for DB: {} entries", result.len());
    let time_spent = after - before;
    log::info!(
        "Took {} seconds to load price DB",
        time_spent.num_milliseconds() as f64 / 1000.
    );
    Ok(())
}

/// Source of the price information.
/// In the DB, latter one (larger one as Ord) has priority,
/// and if you have events with higher priority,
/// lower priority events are discarded.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum PriceSource {
    Ledger,
    PriceDB,
}

/// Builder of [`PriceRepository`].
#[derive(Debug, Default)]
pub(super) struct PriceRepositoryBuilder<'ctx> {
    records: HashMap<
        Commodity<'ctx>,
        HashMap<Commodity<'ctx>, (PriceSource, Vec<(NaiveDate, Decimal)>)>,
    >,
}

/// Event of commodity price.
pub(super) struct PriceEvent<'ctx> {
    // may good to use SingleAmount instead.
    price_of: Commodity<'ctx>,
    price_by: Commodity<'ctx>,
    date: NaiveDate,
    rate: Decimal,
}

impl<'ctx> PriceRepositoryBuilder<'ctx> {
    fn insert_price(&mut self, source: PriceSource, event: PriceEvent<'ctx>) {
        let entry: &mut _ = self
            .records
            .entry(event.price_by)
            .or_default()
            .entry(event.price_of)
            .or_insert((PriceSource::Ledger, Vec::new()));
        if entry.0 < source {
            entry.0 = source;
            entry.1.clear();
        }
        entry.1.push((event.date, event.rate));
        let entry: &mut _ = self
            .records
            .entry(event.price_of)
            .or_default()
            .entry(event.price_by)
            .or_insert((PriceSource::Ledger, Vec::new()));
        if entry.0 < source {
            entry.0 = source;
            entry.1.clear();
        }
        entry.1.push((event.date, Decimal::ONE / event.rate));
    }

    fn build(mut self) -> NaivePriceRepository<'ctx> {
        self.records
            .values_mut()
            .for_each(|x| x.values_mut().for_each(|x| x.1.sort()));
        NaivePriceRepository {
            records: self.records,
        }
    }
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
    pub fn new(inner: NaivePriceRepository<'ctx>) -> Self {
        Self {
            inner,
            cache: HashMap::new(),
        }
    }

    pub fn compute_price(
        &mut self,
        price_of: Commodity<'ctx>,
        price_with: Commodity<'ctx>,
        date: NaiveDate,
    ) -> Option<Decimal> {
        if price_of == price_with {
            return Some(Decimal::ONE);
        }
        self.cache
            .entry((price_with, date))
            .or_insert_with(|| self.inner.compute_price_table(price_with, date))
            .get(&price_of)
            .map(|WithDistance(_, rate)| *rate)
    }
}

#[derive(Debug)]
struct NaivePriceRepository<'ctx> {
    // from comodity -> to commodity -> date -> price.
    // e.g. USD AAPL 2024-01-01 100 means 1 AAPL == 100 USD at 2024-01-01.
    // the value are sorted in NaiveDate order.
    records: HashMap<
        Commodity<'ctx>,
        HashMap<Commodity<'ctx>, (PriceSource, Vec<(NaiveDate, Decimal)>)>,
    >,
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
        self.0.partial_cmp(&other)
    }
}

impl<T: Eq> Ord for WithDistance<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<'ctx> NaivePriceRepository<'ctx> {
    // might be safer to convert eval::SingleAmount into a given commodity.
    pub fn compute_price(
        &self,
        price_of: Commodity<'ctx>,
        price_with: Commodity<'ctx>,
        date: NaiveDate,
    ) -> Option<Decimal> {
        if price_of == price_with {
            return Some(Decimal::ONE);
        }
        match self.compute_price_table(price_with, date).get(&price_of) {
            None => None,
            Some(WithDistance(_, x)) => Some(*x),
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
            for (j, (source, rates)) in match self.records.get(&prev) {
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
