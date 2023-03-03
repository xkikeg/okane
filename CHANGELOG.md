# Change Log

## [Unreleased] - ReleaseDate

### Added

* Added bunch of directives.
    - Added account, commodity directive (#77).
    - Added include directive (GH-74).
    - Added apply tag directive (https://github.com/xkikeg/okane/issues/71).
    - #68 - Added top level comment.

### Changed

* #75 - Made rewrite rule case insensitive, which is more practical.

### Fixed

* #80 - Allowed to use "yyyy-mm-dd" syntax date.
* #81 - Properly handle metadata / apply tag with typed value.

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
