# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
### Changed
- Add support for overloaded contract functions (PR #166)
- Removed `error_chain` and reworked errors manually. Also removes the `backtrace` feature. (#167)
- Update to 2018 edition (PR #172)
### Dependencies
- Upgrade syn, proc-macro2, quote and heck crates (PR #169)

## [10.0.0] - 2020-01-08
### Changed
- Replace docopt by structopt (clap) because of performance issue (#161)
### Fixed
- Return an exit code 1 on failure, including wrong input parameters
