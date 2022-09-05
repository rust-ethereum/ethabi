// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! ABI param and parsing for it.

#[cfg(feature = "full-serde")]
mod lenient;
#[cfg(feature = "full-serde")]
pub use lenient::LenientTokenizer;

#[cfg(feature = "full-serde")]
mod strict;
#[cfg(feature = "full-serde")]
pub use strict::StrictTokenizer;

mod token;
pub use token::Token;

#[cfg(all(feature = "serde", not(feature = "std")))]
use crate::no_std_prelude::*;
#[cfg(feature = "serde")]
use core::cmp::Ordering::{Equal, Less};

#[cfg(feature = "serde")]
use crate::{Error, ParamType};

/// This trait should be used to parse string values as tokens.
#[cfg(feature = "serde")]
pub trait Tokenizer {
	/// Tries to parse a string as a token of given type.
	fn tokenize(param: &ParamType, value: &str) -> Result<Token, Error> {
		match *param {
			ParamType::Address => {
				Self::tokenize_address(value.strip_prefix("0x").unwrap_or(value)).map(|a| Token::Address(a.into()))
			}
			ParamType::String => Self::tokenize_string(value).map(Token::String),
			ParamType::Bool => Self::tokenize_bool(value).map(Token::Bool),
			ParamType::Bytes => Self::tokenize_bytes(value.strip_prefix("0x").unwrap_or(value)).map(Token::Bytes),
			ParamType::FixedBytes(len) => {
				Self::tokenize_fixed_bytes(value.strip_prefix("0x").unwrap_or(value), len).map(Token::FixedBytes)
			}
			ParamType::Uint(_) => Self::tokenize_uint(value).map(Into::into).map(Token::Uint),
			ParamType::Int(_) => Self::tokenize_int(value).map(Into::into).map(Token::Int),
			ParamType::Array(ref p) => Self::tokenize_array(value, p).map(Token::Array),
			ParamType::FixedArray(ref p, len) => Self::tokenize_fixed_array(value, p, len).map(Token::FixedArray),
			ParamType::Tuple(ref p) => Self::tokenize_struct(value, p).map(Token::Tuple),
		}
	}

	/// Tries to parse a value as a vector of tokens of fixed size.
	fn tokenize_fixed_array(value: &str, param: &ParamType, len: usize) -> Result<Vec<Token>, Error> {
		let result = Self::tokenize_array(value, param)?;
		match result.len() == len {
			true => Ok(result),
			false => Err(Error::InvalidData),
		}
	}

	/// Tried to parse a struct as a vector of tokens
	fn tokenize_struct(value: &str, param: &[ParamType]) -> Result<Vec<Token>, Error> {
		if !value.starts_with('(') || !value.ends_with(')') {
			return Err(Error::InvalidData);
		}

		if value.chars().count() == 2 {
			return Ok(vec![]);
		}

		let mut result = vec![];
		let mut nested = 0isize;
		let mut ignore = false;
		let mut last_item = 1;

		let mut array_nested = 0isize;
		let mut array_item_start = 1;
		let mut last_is_array = false;

		let mut params = param.iter();
		for (pos, ch) in value.chars().enumerate() {
			match ch {
				'[' if !ignore => {
					if array_nested == 0 {
						array_item_start = pos;
					}
					array_nested += 1;
				}
				']' if !ignore => {
					array_nested -= 1;

					if nested > 0 {
						// still in nested tuple
						continue;
					}

					match array_nested.cmp(&0) {
						Less => {
							return Err(Error::InvalidData);
						}
						Equal => {
							let sub = &value[array_item_start..pos + 1];
							let token = Self::tokenize(params.next().ok_or(Error::InvalidData)?, sub)?;
							result.push(token);
							last_is_array = !last_is_array;
						}
						_ => {}
					}
				}
				_ if array_nested != 0 => continue,
				'(' if !ignore => {
					nested += 1;
				}
				')' if !ignore && last_is_array => {
					nested -= 1;
					last_is_array = !last_is_array;
				}
				')' if !ignore => {
					nested -= 1;

					match nested.cmp(&0) {
						Less => {
							return Err(Error::InvalidData);
						}
						Equal => {
							if last_is_array {
								last_is_array = !last_is_array;
							} else {
								let sub = &value[last_item..pos];
								let token = Self::tokenize(params.next().ok_or(Error::InvalidData)?, sub)?;
								result.push(token);
								last_item = pos + 1;
							}
						}
						_ => {}
					}
				}
				'"' => {
					ignore = !ignore;
				}
				',' if array_nested == 0 && nested == 1 && !ignore && last_is_array => {
					last_is_array = !last_is_array;
				}
				',' if nested == 1 && !ignore => {
					let sub = &value[last_item..pos];
					let token = Self::tokenize(params.next().ok_or(Error::InvalidData)?, sub)?;
					result.push(token);
					last_item = pos + 1;
				}
				_ => (),
			}
		}

		if ignore {
			return Err(Error::InvalidData);
		}

		Ok(result)
	}

	/// Tries to parse a value as a vector of tokens.
	fn tokenize_array(value: &str, param: &ParamType) -> Result<Vec<Token>, Error> {
		if !value.starts_with('[') || !value.ends_with(']') {
			return Err(Error::InvalidData);
		}

		if value.chars().count() == 2 {
			return Ok(vec![]);
		}

		let mut result = vec![];
		let mut nested = 0isize;
		let mut ignore = false;
		let mut last_item = 1;

		let mut tuple_nested = 0isize;
		let mut tuple_item_start = 1;
		let mut last_is_tuple = false;
		for (i, ch) in value.chars().enumerate() {
			match ch {
				'(' if !ignore => {
					if tuple_nested == 0 {
						tuple_item_start = i;
					}
					tuple_nested += 1;
				}
				')' if !ignore => {
					tuple_nested -= 1;
					match tuple_nested.cmp(&0) {
						Less => {
							return Err(Error::InvalidData);
						}
						Equal => {
							let sub = &value[tuple_item_start..i + 1];
							let token = Self::tokenize(param, sub)?;
							result.push(token);
							last_is_tuple = !last_is_tuple;
						}
						_ => {}
					}
				}
				_ if tuple_nested != 0 => continue,
				'[' if !ignore => {
					nested += 1;
				}
				']' if !ignore && last_is_tuple => {
					nested -= 1;
					last_is_tuple = !last_is_tuple;
				}
				']' if !ignore => {
					nested -= 1;
					match nested.cmp(&0) {
						Less => {
							return Err(Error::InvalidData);
						}
						Equal => {
							if last_is_tuple {
								last_is_tuple = !last_is_tuple;
							} else {
								let sub = &value[last_item..i];
								let token = Self::tokenize(param, sub)?;
								result.push(token);
								last_item = i + 1;
							}
						}
						_ => {}
					}
				}
				'"' => {
					ignore = !ignore;
				}
				',' if tuple_nested == 0 && nested == 1 && !ignore && last_is_tuple => {
					last_is_tuple = !last_is_tuple;
				}
				',' if tuple_nested == 0 && nested == 1 && !ignore => {
					let sub = &value[last_item..i];
					let token = Self::tokenize(param, sub)?;
					result.push(token);
					last_item = i + 1;
				}
				_ => (),
			}
		}

		if ignore {
			return Err(Error::InvalidData);
		}

		Ok(result)
	}

	/// Tries to parse a value as an address.
	fn tokenize_address(value: &str) -> Result<[u8; 20], Error>;

	/// Tries to parse a value as a string.
	fn tokenize_string(value: &str) -> Result<String, Error>;

	/// Tries to parse a value as a bool.
	fn tokenize_bool(value: &str) -> Result<bool, Error>;

	/// Tries to parse a value as bytes.
	fn tokenize_bytes(value: &str) -> Result<Vec<u8>, Error>;

	/// Tries to parse a value as bytes.
	fn tokenize_fixed_bytes(value: &str, len: usize) -> Result<Vec<u8>, Error>;

	/// Tries to parse a value as unsigned integer.
	fn tokenize_uint(value: &str) -> Result<[u8; 32], Error>;

	/// Tries to parse a value as signed integer.
	fn tokenize_int(value: &str) -> Result<[u8; 32], Error>;
}

#[cfg(all(test, feature = "full-serde"))]
mod test {
	use super::{LenientTokenizer, ParamType, Tokenizer};
	use crate::Token;

	#[test]
	fn single_quoted_in_array_must_error() {
		assert!(LenientTokenizer::tokenize_array("[1,\"0,false]", &ParamType::Bool).is_err());
		assert!(LenientTokenizer::tokenize_array("[false\"]", &ParamType::Bool).is_err());
		assert!(LenientTokenizer::tokenize_array("[1,false\"]", &ParamType::Bool).is_err());
		assert!(LenientTokenizer::tokenize_array("[1,\"0\",false]", &ParamType::Bool).is_err());
		assert!(LenientTokenizer::tokenize_array("[1,0]", &ParamType::Bool).is_ok());
	}

	#[test]
	fn tuples_arrays_mixed() {
		assert_eq!(
			LenientTokenizer::tokenize_array(
				"[([(true)],[(false,true)])]",
				&ParamType::Tuple(vec![
					ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Bool]))),
					ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Bool, ParamType::Bool]))),
				]),
			)
			.unwrap(),
			vec![Token::Tuple(vec![
				Token::Array(vec![Token::Tuple(vec![Token::Bool(true)])]),
				Token::Array(vec![Token::Tuple(vec![Token::Bool(false), Token::Bool(true)])]),
			])]
		);

		assert_eq!(
			LenientTokenizer::tokenize_struct(
				"([(true)],[(false,true)])",
				&[
					ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Bool]))),
					ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Bool, ParamType::Bool]))),
				]
			)
			.unwrap(),
			vec![
				Token::Array(vec![Token::Tuple(vec![Token::Bool(true)])]),
				Token::Array(vec![Token::Tuple(vec![Token::Bool(false), Token::Bool(true)])]),
			]
		);
	}

	#[test]
	fn tuple_array_nested() {
		assert_eq!(
			LenientTokenizer::tokenize_struct(
				"([(5c9d55b78febcc2061715ba4f57ecf8ea2711f2c)],2)",
				&[ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Address,],)),), ParamType::Uint(256,),]
			)
			.unwrap(),
			vec![
				Token::Array(vec![Token::Tuple(vec![Token::Address(
					"0x5c9d55b78febcc2061715ba4f57ecf8ea2711f2c".parse().unwrap(),
				),])]),
				Token::Uint(2u64.into()),
			]
		);
	}
}
