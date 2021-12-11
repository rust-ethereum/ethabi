// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Contract error

#[cfg(feature = "full-serde")]
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;
use crate::{
	decode, encode, errors,
	signature::{long_signature, short_signature},
	Bytes, Hash, Param, ParamType, Result, Token,
};

/// Contract error specification.
#[cfg_attr(feature = "full-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
	/// Error name.
	#[cfg_attr(feature = "full-serde", serde(deserialize_with = "crate::util::sanitize_name::deserialize"))]
	pub name: String,
	/// Error input.
	pub inputs: Vec<Param>,
}

impl Error {
	/// Returns types of all params.
	fn param_types(&self) -> Vec<ParamType> {
		self.inputs.iter().map(|p| p.kind.clone()).collect()
	}

	/// Error signature
	pub fn signature(&self) -> Hash {
		long_signature(&self.name, &self.param_types())
	}

	/// Prepares ABI error with given input params.
	pub fn encode(&self, tokens: &[Token]) -> Result<Bytes> {
		let params = self.param_types();

		if !Token::types_check(tokens, &params) {
			return Err(errors::Error::InvalidData);
		}

		let signed = short_signature(&self.name, &params).to_vec();
		let encoded = encode(tokens);
		Ok(signed.into_iter().chain(encoded.into_iter()).collect())
	}

	/// Parses the ABI function input to a list of tokens.
	pub fn decode(&self, data: &[u8]) -> Result<Vec<Token>> {
		decode(&self.param_types(), data)
	}
}
