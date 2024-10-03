use std::path::Path;

use winnow::Parser as _;

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

use std::collections::BTreeMap;

use super::{commodity::Commodity, eval::SingleAmount};

/// `[PriceHistoryStore]` holds the commodity conversion history.
/// Later one can create an instance of `[PriceQuery]`.
pub struct PriceHistoryStore<'arena> {
    events: Vec<PriceEvent<'arena>>,
}

/// Database which allows to query historical price information.
pub struct PriceQuery;

/// Each event of price history.
struct PriceEvent<'arena> {
    value_from: SingleAmount<'arena>,
    value_to: SingleAmount<'arena>,
    event_type: PriceEventType,
}

/// Type of [PriceEvent].
enum PriceEventType {
    /// Provided from external Price DB file.
    PriceDB,
    /// Deduced from the transaction.
    Transaction,
}
