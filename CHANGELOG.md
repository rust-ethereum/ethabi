# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [12.0.0]
### Dependencies
- Upgrade ethereum-types (PR #183)

## [11.0.0] - 2020-01-16
### Changed
- Support overloaded contract functions (PR #166)
- Removed `error_chain` and the `backtrace` feature. (#167)
- Update to 2018 edition (PR #171, #172)
- Fix handling of large ints (PR #173)
- Support Tuple parameter types (structs in Solidity) (PR #174)
### Dependencies
- Upgrade syn, proc-macro2, quote and heck crates (PR #169)

## [10.0.0] - 2020-01-08
### Changed
- Replace docopt by structopt (clap) because of performance issue (#161)
### Fixed
- Return an exit code 1 on failure, including wrong input parameters

