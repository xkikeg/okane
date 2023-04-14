# Change Log

## [Unreleased] - ReleaseDate

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
