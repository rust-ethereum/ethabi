// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Contract constructor call builder.

#[cfg(feature = "full-serde")]
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;
use crate::{encode, Bytes, Error, Param, ParamType, Result, Token};

/// Contract constructor specification.
#[cfg_attr(feature = "full-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Constructor {
	/// Constructor input.
	pub inputs: Vec<Param>,
}

impl Constructor {
	/// Returns all input params of given constructor.
	fn param_types(&self) -> Vec<ParamType> {
		self.inputs.iter().map(|p| p.kind.clone()).collect()
	}

	/// Prepares ABI constructor call with given input params.
	pub fn encode_input(&self, code: Bytes, tokens: &[Token]) -> Result<Bytes> {
		let params = self.param_types();

		if Token::types_check(tokens, &params) {
			Ok(code.into_iter().chain(encode(tokens)).collect())
		} else {
			Err(Error::InvalidData)
		}
	}
}
