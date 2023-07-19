// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;
use crate::{Error, ParamType};

/// Used to convert param type represented as a string to rust structure.
pub struct Reader;

impl Reader {
	/// Converts string to param type.
	pub fn read(name: &str) -> Result<ParamType, Error> {
		let name = name.trim();
	
		if name.starts_with('(') && name.ends_with(')') {
			let r = Reader::read_tuple(&name[1..name.len() - 1]);
			return r;
		}
	
		if name.ends_with(']') {
			return Reader::read_array(name);
		}
	
		Reader::read_primitive(name)
	}

	fn read_tuple(s: &str) -> Result<ParamType, Error> {
		// println!("read_tuple: {}", s);
		let mut subtypes = Vec::new();
		let mut start = 0;
		let mut parens = 0;
	
		for (i, c) in s.chars().enumerate() {
			match c {
				'(' => parens += 1,
				')' => parens -= 1,
				',' if parens == 0 => {
					subtypes.push(Reader::read(&s[start..i])?);
					start = i + 1;
				}
				_ => (),
			}
		}
		// Add the last subtype.
		subtypes.push(Reader::read(&s[start..])?);
		Ok(ParamType::Tuple(subtypes))
	}
	fn read_array(s: &str) -> Result<ParamType, Error> {
		// find the matching opening bracket
		let mut brackets = 0;
		let mut i = s.len() - 1;
		for c in s.chars().rev() {
			match c {
				']' => brackets += 1,
				'[' => {
					brackets -= 1;
					if brackets == 0 {
						break;
					}
				}
				_ => (),
			}
			i -= 1;
		}
	
		let size_str = &s[i + 1..s.len() - 1];
		let subtype = Reader::read(&s[..i])?;
		let size = if size_str.is_empty() {
			None // dynamic array
		} else {
			Some(size_str.parse().map_err(Error::ParseInt)?)
		};
	
		match size {
			Some(len) => Ok(ParamType::FixedArray(Box::new(subtype), len)),
			None => Ok(ParamType::Array(Box::new(subtype))),
		}
	}
	
	fn read_primitive(s: &str) -> Result<ParamType, Error> {
		match s {
			"address" => Ok(ParamType::Address),
			"bytes" => Ok(ParamType::Bytes),
			"bool" => Ok(ParamType::Bool),
			"string" => Ok(ParamType::String),
			"int" => Ok(ParamType::Int(256)),
			"tuple" => Ok(ParamType::Tuple(vec![])),
			"uint" => Ok(ParamType::Uint(256)),
			_ => {
				if s.starts_with("int") {
					let len = s[3..].parse().map_err(Error::ParseInt)?;
					Ok(ParamType::Int(len))
				} else if s.starts_with("uint") {
					let len = s[4..].parse().map_err(Error::ParseInt)?;
					Ok(ParamType::Uint(len))
				} else if s.starts_with("bytes") {
					let len = s[5..].parse().map_err(Error::ParseInt)?;
					Ok(ParamType::FixedBytes(len))
				} else {
					Ok(ParamType::Uint(8)) // fallback
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Reader;
	#[cfg(not(feature = "std"))]
	use crate::no_std_prelude::*;
	use crate::ParamType;

	#[test]
	fn test_read_param() {
		assert_eq!(Reader::read("address").unwrap(), ParamType::Address);
		assert_eq!(Reader::read("bytes").unwrap(), ParamType::Bytes);
		assert_eq!(Reader::read("bytes32").unwrap(), ParamType::FixedBytes(32));
		assert_eq!(Reader::read("bool").unwrap(), ParamType::Bool);
		assert_eq!(Reader::read("string").unwrap(), ParamType::String);
		assert_eq!(Reader::read("int").unwrap(), ParamType::Int(256));
		assert_eq!(Reader::read("uint").unwrap(), ParamType::Uint(256));
		assert_eq!(Reader::read("int32").unwrap(), ParamType::Int(32));
		assert_eq!(Reader::read("uint32").unwrap(), ParamType::Uint(32));
	}

	#[test]
	fn test_read_array_param() {
		assert_eq!(Reader::read("address[]").unwrap(), ParamType::Array(Box::new(ParamType::Address)));
		assert_eq!(Reader::read("uint[]").unwrap(), ParamType::Array(Box::new(ParamType::Uint(256))));
		assert_eq!(Reader::read("bytes[]").unwrap(), ParamType::Array(Box::new(ParamType::Bytes)));
		assert_eq!(
			Reader::read("bool[][]").unwrap(),
			ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Bool))))
		);
	}

	#[test]
	fn test_read_fixed_array_param() {
		assert_eq!(Reader::read("address[2]").unwrap(), ParamType::FixedArray(Box::new(ParamType::Address), 2));
		assert_eq!(Reader::read("bool[17]").unwrap(), ParamType::FixedArray(Box::new(ParamType::Bool), 17));
		assert_eq!(
			Reader::read("bytes[45][3]").unwrap(),
			ParamType::FixedArray(Box::new(ParamType::FixedArray(Box::new(ParamType::Bytes), 45)), 3)
		);
	}

	#[test]
	fn test_read_mixed_arrays() {
		assert_eq!(
			Reader::read("bool[][3]").unwrap(),
			ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 3)
		);
		assert_eq!(
			Reader::read("bool[3][]").unwrap(),
			ParamType::Array(Box::new(ParamType::FixedArray(Box::new(ParamType::Bool), 3)))
		);
	}

	#[test]
	fn test_read_struct_param() {
		assert_eq!(
			Reader::read("(address,bool)").unwrap(),
			ParamType::Tuple(vec![ParamType::Address, ParamType::Bool])
		);
		assert_eq!(
			Reader::read("(bool[3],uint256)").unwrap(),
			ParamType::Tuple(vec![ParamType::FixedArray(Box::new(ParamType::Bool), 3), ParamType::Uint(256)])
		);
	}

	#[test]
	fn test_read_nested_struct_param() {
		assert_eq!(
			Reader::read("(address,bool,(bool,uint256))").unwrap(),
			ParamType::Tuple(vec![
				ParamType::Address,
				ParamType::Bool,
				ParamType::Tuple(vec![ParamType::Bool, ParamType::Uint(256)])
			])
		);
	}

	#[test]
	fn test_read_complex_nested_struct_param() {
		assert_eq!(
			Reader::read("(address,bool,(bool,uint256,(bool,uint256)),(bool,uint256))").unwrap(),
			ParamType::Tuple(vec![
				ParamType::Address,
				ParamType::Bool,
				ParamType::Tuple(vec![
					ParamType::Bool,
					ParamType::Uint(256),
					ParamType::Tuple(vec![ParamType::Bool, ParamType::Uint(256)])
				]),
				ParamType::Tuple(vec![ParamType::Bool, ParamType::Uint(256)])
			])
		);
	}
	#[test]
	fn test_read_complex_nested_struct_param2() {
		assert_eq!(
			Reader::read("(((string,uint256),int256),uint256,int256)").unwrap(),
			ParamType::Tuple(vec![
				ParamType::Tuple(vec![
					ParamType::Tuple(vec![ParamType::String, ParamType::Uint(256)]),
					ParamType::Int(256)
				]),
				ParamType::Uint(256),
				ParamType::Int(256)
			])
		);
	}

	#[test]
	fn test_read_nested_tuple_array_param() {
		assert_eq!(
			Reader::read("(uint256,bytes32)[]").unwrap(),
			ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Uint(256), ParamType::FixedBytes(32)])))
		)
	}

	#[test]
	fn test_read_inner_tuple_array_param() {
		use crate::param_type::Writer;
		let abi = "((uint256,bytes32)[],address)";
		let read = Reader::read(abi).unwrap();

		let param = ParamType::Tuple(vec![
			ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Uint(256), ParamType::FixedBytes(32)]))),
			ParamType::Address,
		]);

		assert_eq!(read, param);

		assert_eq!(abi, Writer::write(&param));
	}
}
