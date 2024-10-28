# Change Log

## [Unreleased] - ReleaseDate

### Added

### Changed

### Fixed

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
