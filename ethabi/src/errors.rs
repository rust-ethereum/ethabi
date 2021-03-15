// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(not(feature = "std"))]
use alloc::string::{self, String};
use anyhow::anyhow;
use core::num;
#[cfg(feature = "std")]
use std::string;
#[cfg(feature = "std")]
use thiserror::Error;

/// Ethabi result type
pub type Result<T> = core::result::Result<T, Error>;

/// Ethabi errors
#[cfg(feature = "std")]
#[derive(Debug, Error)]
pub enum Error {
	/// Invalid entity such as a bad function name.
	#[error("Invalid name: {0}")]
	InvalidName(String),
	/// Invalid data.
	#[error("Invalid data")]
	InvalidData,
	/// Serialization error.
	#[error("Serialization error: {0}")]
	SerdeJson(#[from] serde_json::Error),
	/// Integer parsing error.
	#[error("Integer parsing error: {0}")]
	ParseInt(#[from] num::ParseIntError),
	/// UTF-8 parsing error.
	#[error("UTF-8 parsing error: {0}")]
	Utf8(#[from] string::FromUtf8Error),
	/// Hex string parsing error.
	#[error("Hex parsing error: {0}")]
	Hex(#[from] hex::FromHexError),
	/// Other errors.
	#[error("{0}")]
	Other(#[from] anyhow::Error),
}

/// Ethabi `no_std` errors
#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub enum Error {
	/// Invalid entity such as a bad function name.
	InvalidName(String),
	/// Invalid data.
	InvalidData,
	/// Integer parsing error.
	ParseInt(num::ParseIntError),
	/// UTF-8 parsing error.
	Utf8(string::FromUtf8Error),
	/// Hex string parsing error.
	Hex(hex::FromHexError),
	/// Other errors.
	Other(anyhow::Error),
}

#[cfg(not(feature = "std"))]
impl From<num::ParseIntError> for Error {
	fn from(err: num::ParseIntError) -> Self {
		Self::ParseInt(err)
	}
}

#[cfg(not(feature = "std"))]
impl From<string::FromUtf8Error> for Error {
	fn from(err: string::FromUtf8Error) -> Self {
		Self::Utf8(err)
	}
}

#[cfg(not(feature = "std"))]
impl From<hex::FromHexError> for Error {
	fn from(err: hex::FromHexError) -> Self {
		Self::Hex(err)
	}
}

#[cfg(not(feature = "std"))]
impl From<anyhow::Error> for Error {
	fn from(err: anyhow::Error) -> Self {
		Self::Other(err)
	}
}

impl From<uint::FromDecStrErr> for Error {
	fn from(err: uint::FromDecStrErr) -> Self {
		use uint::FromDecStrErr::*;
		match err {
			InvalidCharacter => anyhow!("Uint parse error: InvalidCharacter"),
			InvalidLength => anyhow!("Uint parse error: InvalidLength"),
		}
		.into()
	}
}
