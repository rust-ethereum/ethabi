# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [Unreleased]
### Changed
- Removed `error_chain` and reworked errors manually (#167)

## [10.0.0] - 2020-01-08
### Changed
- Replace docopt by structopt (clap) because of performance issue (#161)
### Fixed
- Return an exit code 1 on failure, including wrong input parameters
### Dependencies
