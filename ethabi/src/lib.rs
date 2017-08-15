//! Ethereum ABI encoding decoding library.

#![warn(missing_docs)]

extern crate rustc_hex as hex;
extern crate serde;
extern crate serde_json;
extern crate tiny_keccak;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

pub mod spec;
pub mod token;
mod constructor;
mod contract;
mod decoder;
mod encoder;
mod errors;
mod event;
mod filter;
mod function;
mod log;
mod param;
mod signature;
pub mod util;

pub use spec::Interface;
pub use constructor::Constructor;
pub use contract::{Contract, Functions, Events};
pub use token::Token;
pub use errors::{Error, ErrorKind, Result, ResultExt};
pub use encoder::Encoder;
pub use decoder::Decoder;
pub use function::Function;
pub use param::Param;
pub use log::Log;
pub use event::{Event, LogParam};

/// ABI address.
pub type Address = [u8; 20];

/// ABI fixed bytes.
pub type FixedBytes = Vec<u8>;

/// ABI bytes.
pub type Bytes = Vec<u8>;

/// ABI signed integer.
pub type Int = [u8; 32];

/// ABI unsigned integer.
pub type Uint = [u8; 32];

/// Commonly used FixedBytes of size 32
pub type Hash = [u8; 32];
