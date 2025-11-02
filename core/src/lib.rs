//! okane-core crate is the library to support
//! [okane](https://crates.io/crates/okane) CLI tool functionality,
//! withreusable components.

pub mod format;
pub mod load;
pub mod parse;
pub mod report;
pub mod syntax;
pub(crate) mod testing;
pub mod utility;

#[cfg(test)]
#[ctor::ctor]
fn unit_test_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::max())
        .try_init();
}
