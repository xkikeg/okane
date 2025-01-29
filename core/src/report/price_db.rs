use std::path::Path;

use crate::parse;

#[derive(Debug, thiserror::Error)]
pub enum PriceDBError {
    #[error("failed to perform IO")]
    IO(#[from] std::io::Error),
    #[error("failed to parse price DB entry: {0}")]
    Parse(#[from] parse::ParseError),
}

/// Loads PriceDB information from the given file.
pub fn load_price_db(path: &Path) -> Result<(), PriceDBError> {
    // Even though price db can be up to a few megabytes,
    // still it's much easier to load everything into memory.
    let before = chrono::Local::now();
    let content = std::fs::read_to_string(path)?;
    let mut num = 0;
    for entry in parse::price::parse_price_db(&parse::ParseOptions::default(), &content) {
        entry?;
        num += 1;
    }
    let after = chrono::Local::now();
    log::info!("TODO: use this for DB: {} entries", num);
    let time_spent = after - before;
    log::info!(
        "Took {} seconds to load price DB",
        time_spent.num_milliseconds() as f64 / 1000.
    );
    Ok(())
}
