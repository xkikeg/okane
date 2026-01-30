# Change Log

## [Unreleased] - ReleaseDate

### Added

### Changed

### Fixed

## [0.18.0] - 2026-01-30

### Added

* CLI: Now `import` supports tsv file (https://github.com/xkikeg/okane/pull/302).
* CLI: `balance` and `register` supports `--current` flag (https://github.com/xkikeg/okane/pull/312).
* Core: Improved error message for report API (https://github.com/xkikeg/okane/pull/316, https://github.com/xkikeg/okane/pull/318).
* CLI: Now `import` config supports `template` field (https://github.com/xkikeg/okane/pull/326). \
  This clarifies the config is only meant for template, not a final config for `import`.

### Changed

* Both: Updated to edition 2024, minimum version 1.86 (https://github.com/xkikeg/okane/pull/309).
* Core: Renamed `UpToDate` strategy field (https://github.com/xkikeg/okane/pull/315).
* Core: `report::Amount` now uses `BTreeMap` under the hood (https://github.com/xkikeg/okane/pull/317).
* CLI: Changed `okane --version` to return a long version (https://github.com/xkikeg/okane/pull/285).
* CLI: Introduced `format.file_type` (https://github.com/xkikeg/okane/pull/329). \
  Now you **need** to specify `format.file_type` to `ISO_CAMT053` or `VISECA` for corresponding formats. \
  CSV and TSV files are still automatically inferred, however, user can also set the config value to `CSV` / `TSV` if they want.
* CLI: Made all module private (https://github.com/xkikeg/okane/pull/332).
* Core: Reordered `*Amount` method args (https://github.com/xkikeg/okane/pull/337).
* CLI: Viseca format supports transaction without category (https://github.com/xkikeg/okane/pull/341).
* Core: `Balance::get` to accept `Account` value, not `&Account` reference (https://github.com/xkikeg/okane/pull/344).
* Core: Added new APIs `Balance::{into,from}_map` (https://github.com/xkikeg/okane/pull/345).
* Core: Added `Balance::from_iter`, repurposed `Balance::from_values` (https://github.com/xkikeg/okane/pull/346).

### Fixed

* CLI: Revert warning logs on unmatched transaction import (https://github.com/xkikeg/okane/pull/322).
* Other: Fixed bug in syntax documentation (https://github.com/xkikeg/okane/pull/325).
* Both: Fixed amount printing on negative amount with multi commodities (https://github.com/xkikeg/okane/pull/334).

## [0.17.0] - 2025-12-25

### Added

* CLI: import balances posting (https://github.com/xkikeg/okane/pull/288).

### Changed

* CLI: Supports more ISO Camt053 related to `<AmtDtls>` (https://github.com/xkikeg/okane/pull/290).
* CLI: Removed duplciated integration test (https://github.com/xkikeg/okane/pull/293).
* CLI: Added `commodity` matcher (https://github.com/xkikeg/okane/pull/301, https://github.com/xkikeg/okane/pull/303).
* CLI: Supports `secondary_commodity` matcher for all formats (https://github.com/xkikeg/okane/pull/296, https://github.com/xkikeg/okane/pull/297).
* CLI: Added hidden fee support (https://github.com/xkikeg/okane/pull/298).

### Fixed

* meta: Add clippy CI (https://github.com/xkikeg/okane/pull/286).
* CLI: refactored extract logic (https://github.com/xkikeg/okane/pull/292, https://github.com/xkikeg/okane/pull/295).

## [0.16.0] - 2025-11-02

### Added

* CLI: CSV import can have `code` column (https://github.com/xkikeg/okane/pull/270).

### Changed

* Core: Removed `PrettyDecimal`, now published as independent crate (https://github.com/xkikeg/okane/pull/258).
* Core: Use dunce filepath canonicalization for clearer path on Windows (https://github.com/xkikeg/okane/pull/272).
* CLI: Renamed `format.commodity` option into `output.commodity` (https://github.com/xkikeg/okane/pull/260). \
  This allows emitting comma separated decimal on `import` command as well.
* CLI: Factored out `OneBasedIndex` as a separate crate (https://github.com/xkikeg/okane/pull/265). \
  Now it's https://crates.io/crates/one-based.
* Core: Use dense commodty for performance (https://github.com/xkikeg/okane/pull/280).

### Fixed

* CLI: Enabled LTO on release build (https://github.com/xkikeg/okane/pull/261).
* Core: Allow double-alias (https://github.com/xkikeg/okane/pull/274).
* CLI: Fixed `import` on Windows (https://github.com/xkikeg/okane/pull/276).

## [0.15.0] - 2025-06-19

### Added

* Improved error message on `balance` (https://github.com/xkikeg/okane/pull/246).
* Added `import` option to rename commodity (https://github.com/xkikeg/okane/pull/253).

### Fixed

* Test fix on Windows (https://github.com/xkikeg/okane/pull/247).
* Fixed a bug in PrettyDecimal crashing in comma3dot mode with fraction (https://github.com/xkikeg/okane/pull/249).

## [0.14.0] - 2025-04-08

### Added

* `balance` supports commodity conversion.
    * https://github.com/xkikeg/okane/pull/201
    * and a few other PRs.
* `eval` command to try commodity conversion (https://github.com/xkikeg/okane/pull/210).
* Tool supports glob include.
    * https://github.com/xkikeg/okane/pull/205
    * https://github.com/xkikeg/okane/pull/224
    * https://github.com/xkikeg/okane/pull/237

### Changed

* Print some errors in more friendly manner (https://github.com/xkikeg/okane/pull/213).
* `import` emits debits earlier than credits (https://github.com/xkikeg/okane/pull/228).
* `import` applies commodity conversion by default under certain conditions (https://github.com/xkikeg/okane/pull/199).
* `import` supports more variants for ISO Camt053 files (https://github.com/xkikeg/okane/pull/197).
* `import` emits Ledger comment out of CSV note field (https://github.com/xkikeg/okane/pull/192).

### Fixed

* Fixed the issue with non ASCII accounts (https://github.com/xkikeg/okane/pull/221).
* Rejects the commodity rate the same as the main amount (https://github.com/xkikeg/okane/pull/217).
* `import` correctly emits exchange rate for the charges (https://github.com/xkikeg/okane/pull/198).

## [0.13.0] - 2024-10-28

### Added

* Added positional tracking to `syntax` types (https://github.com/xkikeg/okane/pull/183).
* Supported int column index for template field (https://github.com/xkikeg/okane/pull/191).

### Changed

* **IMPORTANT** Use 1-based index for CSV columns (https://github.com/xkikeg/okane/pull/190).
* Enhanced error message on semantic errors (https://github.com/xkikeg/okane/pull/186).
* Renamed `repl` module to `syntax` (https://github.com/xkikeg/okane/pull/184).
* Removed `datamodel` module (https://github.com/xkikeg/okane/pull/181, https://github.com/xkikeg/okane/pull/182).

### Fixed

## [0.12.0] - 2024-09-03

### Added

* Supported commission / charge in CSV import (https://github.com/xkikeg/okane/pull/168).

### Changed

* Changed to correctly assert the in-transaction balance (https://github.com/xkikeg/okane/pull/176).

### Fixed

* Fixed minor lifetime fix (https://github.com/xkikeg/okane/pull/170).
* Fixed memory leak in `okane_core::report::process` (https://github.com/xkikeg/okane/pull/172).
* Fixed balance deduction logic with balance assertion (https://github.com/xkikeg/okane/pull/173).
* Fixed internal logic to avoid future potential inconsistency (https://github.com/xkikeg/okane/pull/177).

## [0.11.0] - 2024-08-07

### Added

* Supported account / commodity aliases (https://github.com/xkikeg/okane/pull/157).
* Supported CSV with $ prefixed amount (https://github.com/xkikeg/okane/pull/159).
* Supported template field in CSV import (https://github.com/xkikeg/okane/pull/163).
* Supported ISO Camt053 2019 edition (https://github.com/xkikeg/okane/pull/165).

### Changed

* Changed conversion logic in import rewrite (https://github.com/xkikeg/okane/pull/162).

### Fixed

* Print the line number on parse failure (https://github.com/xkikeg/okane/pull/153).
* Print the transaction on failed balance (https://github.com/xkikeg/okane/pull/155).


## [0.10.0] - 2024-07-05

### Added

* Added `balance` and `register` CLI command with limited functionality (https://github.com/xkikeg/okane/pull/147).
* Added `accounts` CLI command to list all accounts (https://github.com/xkikeg/okane/pull/128).
* Added `primitve flatten` CLI command to resolve include statement (https://github.com/xkikeg/okane/pull/127).

### Changed

* User can shorten command, such as reg instead of register (https://github.com/xkikeg/okane/pull/132).
* Zero-copy parser (https://github.com/xkikeg/okane/pull/134, https://github.com/xkikeg/okane/pull/136).
* Pretty printing error report (https://github.com/xkikeg/okane/pull/141).

### Fixed

* Fixed ClearState parse at posting (https://github.com/xkikeg/okane/pull/129).
* Fixed benchmark input (https://github.com/xkikeg/okane/pull/133).

## [0.9.0] - 2024-05-12

### Added

* Supported ISO Camt053 DAJT (Debit Adjustment) code (https://github.com/xkikeg/okane/pull/117).

### Changed

* Renamed `equivalent_amount` to `secondary_amount`, with `secondary_commodity` (https://github.com/xkikeg/okane/pull/111).

### Fixed

* Fixed the issue that amount is presented in a negative value by mistake (https://github.com/xkikeg/okane/pull/112).
* Fixed the `format` command to properly accept contiguous transactions (https://github.com/xkikeg/okane/pull/116).

## [0.8.0] - 2024-02-03

### Changed

* Renamed `equivalent_absolute` to `equivalent_amount` (https://github.com/xkikeg/okane/pull/109).

### Fixed

* Fixed import with commodity conversion spec (https://github.com/xkikeg/okane/pull/108).

## [0.7.0] - 2023-11-16

### Added

* Supported comma separated amount to be formatted (https://github.com/xkikeg/okane/pull/99).
* Supported commodity format sub-directive (https://github.com/xkikeg/okane/pull/102).
* Supported `Air-*:` annotations in viseca format (https://github.com/xkikeg/okane/pull/98).

### Changed

* Exposed more fields in `repl` structs (https://github.com/xkikeg/okane/pull/97).
* Unify `repl::Amount` into `repl::expr::Amount` (https://github.com/xkikeg/okane/pull/97).

## [0.6] - 2023-04-14

### Changed

* Factored out core library as an independent crate (https://github.com/xkikeg/okane/pull/90).

## [0.5.4] - 2023-03-03

### Added

* Added bunch of directives.
    - Added account, commodity directive (https://github.com/xkikeg/okane/issues/77).
    - Added include directive (https://github.com/xkikeg/okane/issues/74).
    - Added apply tag directive (https://github.com/xkikeg/okane/issues/71).
    - Added top level comment (https://github.com/xkikeg/okane/issues/68).

### Changed

* Made rewrite rule case insensitive, which is more practical (https://github.com/xkikeg/okane/issues/75).

### Fixed

* Allowed to use "yyyy-mm-dd" syntax date (https://github.com/xkikeg/okane/issues/80).
* Properly handle metadata / apply tag with typed value (https://github.com/xkikeg/okane/issues/81).

## [0.5.3] - 2022-12-15
### Added
* Added ja-JP README.

### Fixed
* Dependencies updated to the latest version.

## [0.5.2] - yyyy-mm-dd

### Added
* Properly handles lot information in the format command.
* Adds configuration to skip initial lines for importing CSV files.

### Fixed
* Adds `STDO` (standing order) subfamily codes in ISO Camt053.
