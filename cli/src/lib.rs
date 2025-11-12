//! okane is a tool to have a plain text accounting, inspired by [Ledger CLI](https://ledger-cli.org/).
//!
//! For the detailed usage, see [README](https://github.com/xkikeg/okane/blob/main/README.md) or
//! [Japanese README](https://github.com/xkikeg/okane/blob/main/README.md).
//!
//! As a library, [`okane-core`](https://crates.io/crates/okane-core) provides reusable functionalities.
//! As oppose to that, this library mainly provides binary specific functionalities, mainly for integration tests.

pub mod cmd;
pub mod format;
pub mod import;
#[cfg(test)]
pub mod one_based_macro;

use shadow_rs::shadow;

shadow!(build);
