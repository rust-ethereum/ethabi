# Changelog

The format is based on [Keep a Changelog].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/

## [18.0.0] - 2022-11-16
### Added
- Decode function that fails if there is leftover data.

### Changed
- Allow serde in no_std.
- Bump ethereum-types to 0.14.0.
- Updated Rust edition to 2021.

### Fixed
- Nested tuple handling.

### Dependencies
- Bump ethereum-types to 0.14.0

## [17.2.0] - 2022-08-12
### Added
- Add `Token::into_tuple`
- Parse nested tuples/arrays

## [17.1.0] - 2022-06-15
### Added
- Add Serialize trait to Token, Log, LogParams.

### Changed
- Optimize encoder to minimize allocations and copying for ~85% speedup.

## [17.0.0] - 2022-03-01
### Added
- `parity-codec` cargo feature.

### Changed
- Make deprecated function "constant" field optional.
- Allow LenientTokenizer to parse uint256 with units `ether|gwei|nano|nanoether|wei`.
- Bump ethereum-types to 0.13.1.

### Fixed
- Make abi of public library function containing enum parameter parsable.

## [16.0.0] - 2021-12-18
### Added
- Reexport signature functions
- Support solidity error type

### Changed
- Remove anyhow in crates intended to be libraries.

### Fixed
- Fix out of bounds access resulting in panic during decoding of dynamic arrays.

## [15.0.0] - 2021-09-23
### Added
- Optional field internalType to Param.
- Support for no std.
- Method to retrieve the short 4 byte signature of a function.

### Changed
- Bump ethereum-types to 0.12.0.

## [14.1.0] - 2021-07-08
### Added
- `Serialize` support for contracts.

### Fixed
- Fix encoding of nested static tuples in dynamic tuple.
- Fix running out of memory when decoding corrupted array encodings.

## [14.0.0] - 2021-03-31
### Added
- Re-export of ethereum-types.
- Support abiv2 tuples.
- Parse StateMutability in Function abis.
- Support the receive function as an additional operation type.

### Changed
- Update ethereum-types dependency.
- Use lossy UTF-8 decoding for strings.

### Deprecated
- Deprecate `Function::constant`.

### Fixed
- Fix Contract having a receive function by default.
- Fix decoder to handle encoded data with length % 32 != 0.

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
