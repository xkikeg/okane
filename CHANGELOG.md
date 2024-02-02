# Change Log

## [Unreleased] - ReleaseDate

### Added

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
