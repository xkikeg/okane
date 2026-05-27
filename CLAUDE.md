# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## About

`okane` is a plain-text accounting tool written in Rust, compatible with [ledger-cli](https://www.ledger-cli.org/) syntax. It is a Cargo workspace with two crates:

- **`core/`** (`okane-core`) — reusable library: parsing, syntax AST, formatting, file loading, and reporting engine.
- **`cli/`** (`okane`) — binary: CLI interface using `clap`, plus import logic for CSV and ISO Camt053 XML.

## Commands

```bash
# Build
cargo build
cargo check

# Run all tests
cargo test

# Run tests for a single crate
cargo test -p okane-core
cargo test -p okane

# Run a single test by name
cargo test <test_name>

# Lint
cargo clippy

# Benchmarks (criterion)
cargo bench

# Run the CLI during development
cargo run -- <subcommand> [args]
RUST_LOG=info cargo run -- import --config path/to/config.yml path/to/input.csv
```

Minimum supported Rust version: **1.88.0**.

## Architecture

### Data flow

```
Ledger files → load::Loader (handles `include` recursion)
             → parse::parse_ledger (winnow parser)
             → syntax::LedgerEntry AST
             → report::process (book-keeping / evaluation)
             → Balance / Register / Accounts output
```

For import: external formats (CSV, ISO Camt053, Viseca) → `cli/src/import/` → `syntax::LedgerEntry` → formatted Ledger output.

### `core/src/syntax/`

The AST. Top-level type is `LedgerEntry<'i, Deco>`, parameterized over a lifetime `'i` (zero-copy string slices into the original file) and a `Decoration` type.

- **`decoration.rs`** — `Decoration` trait controls what extra data is attached to each AST node. Two implementations exist: `plain` (bare identity, used for most processing) and `tracked` (carries source span info, used during parsing when location is needed).
- **`expr.rs`** — Numeric expression types (`Amount`, `ValueExpr`).

### `core/src/parse/`

Winnow-based recursive-descent parser. `parse::parse_ledger()` is the public entry point, returning an iterator of `LedgerEntry`. Sub-modules handle directives, postings, metadata, and expressions.

### `core/src/load.rs`

`Loader<F: FileSystem>` reads a file and resolves `include` directives recursively. A `FakeFileSystem` impl is provided for testing (avoids touching real disk). Arena-allocated via `bumpalo`.

### `core/src/report/`

Evaluation engine. `report::process()` (via `book_keeping`) walks the loaded entries and maintains running balances per account and commodity. Exposes `Balance`, `Account`, `Commodity`, `Transaction`, `Posting` for query and display.

### `cli/src/import/`

Converts external file formats to Ledger entries. Config is a YAML file (`config::ConfigSet`) that maps source files to import rules. Supported formats: CSV (with template expansion), ISO Camt053 XML, Viseca. The `Importer` struct selects the right parser from config and emits formatted Ledger text.

## Testing

- **Golden tests** use the `okane-golden` crate. Golden files live under `testdata/report/golden/` (for balance/register) and `testdata/error/*.error.txt` (for error output). Run with `BLESS=1 cargo test` to regenerate golden files when output intentionally changes.
- **Parameterized tests** use `rstest` with `#[files(...)]` to iterate over `testdata/` inputs automatically.
- **`load::FakeFileSystem`** is the in-memory filesystem for unit tests that exercise loading/parsing without real I/O.
- `RUST_LOG=info` (or `max`) enables logging during test runs; the test harness initializes `env_logger` via `#[ctor]`.
