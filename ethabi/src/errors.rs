// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "std")]
use thiserror::Error;

#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;

/// Ethabi result type
pub type Result<T> = core::result::Result<T, Error>;

/// Ethabi errors
#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug)]
pub enum Error {
	/// Invalid entity such as a bad function name.
	#[cfg_attr(feature = "std", error("Invalid name: {0}"))]
	InvalidName(String),
	/// Invalid data.
	#[cfg_attr(feature = "std", error("Invalid data"))]
	InvalidData,
}
