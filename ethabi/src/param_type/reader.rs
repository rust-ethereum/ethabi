use {ParamType, Error, ErrorKind};

/// Used to convert param type represented as a string to rust structure.
pub struct Reader;

impl Reader {
	/// Converts string to param type.
	pub fn read(name: &str) -> Result<ParamType, Error> {
		// check if it is a fixed or dynamic array.
		match name.chars().last() {
			Some(']') => {
				// take number part
				let num: String = name.chars()
					.rev()
					.skip(1)
					.take_while(|c| *c != '[')
					.collect::<String>()
					.chars()
					.rev()
					.collect();

				let count = name.chars().count();
				if num.len() == 0 {
					// we already know it's a dynamic array!
					let subtype = try!(Reader::read(&name[..count - 2]));
					return Ok(ParamType::Array(Box::new(subtype)));
				} else {
					// it's a fixed array.
					let len = try!(usize::from_str_radix(&num, 10));
					let subtype = try!(Reader::read(&name[..count - num.len() - 2]));
					return Ok(ParamType::FixedArray(Box::new(subtype), len));
				}
			}
			Some(')') => {
				if name.chars().next() == Some('(') {
					let mut subtypes = Vec::new();
					let mut nested = 0isize;
					let mut last_item = 1;

					for (pos, c) in name.chars().enumerate() {
						match c {
							'(' => {
								nested += 1;
							}
							')' => {
								nested -= 1;
								if nested < 0 {
									return Err(ErrorKind::InvalidName(name.to_owned()).into());
								} else if nested == 0 {
									let sub = &name[last_item..pos];
									let subtype = Reader::read(sub)?;
									subtypes.push(subtype);
									last_item = pos + 1;
								}
							}
							',' if nested == 1 => {
								let sub = &name[last_item..pos];
								let subtype = Reader::read(sub)?;
								subtypes.push(subtype);
								last_item = pos + 1;
							}
							_ => ()
						}
					}
					return Ok(ParamType::Tuple(subtypes));
				}

			}
			_ => ()
		}

		let result = match name {
			"address" => ParamType::Address,
			"bytes" => ParamType::Bytes,
			"bool" => ParamType::Bool,
			"string" => ParamType::String,
			"int" => ParamType::Int(256),
			"uint" => ParamType::Uint(256),
			s if s.starts_with("int") => {
				let len = try!(usize::from_str_radix(&s[3..], 10));
				ParamType::Int(len)
			},
			s if s.starts_with("uint") => {
				let len = try!(usize::from_str_radix(&s[4..], 10));
				ParamType::Uint(len)
			},
			s if s.starts_with("bytes") => {
				let len = try!(usize::from_str_radix(&s[5..], 10));
				ParamType::FixedBytes(len)
			},
			_ => {
				return Err(ErrorKind::InvalidName(name.to_owned()).into());
			}
		};

		Ok(result)
	}
}

#[cfg(test)]
mod tests {
	use ParamType;
	use super::Reader;

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
		assert_eq!(Reader::read("bool[][]").unwrap(), ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Bool)))));
	}

	#[test]
	fn test_read_fixed_array_param() {
		assert_eq!(Reader::read("address[2]").unwrap(), ParamType::FixedArray(Box::new(ParamType::Address), 2));
		assert_eq!(Reader::read("bool[17]").unwrap(), ParamType::FixedArray(Box::new(ParamType::Bool), 17));
		assert_eq!(Reader::read("bytes[45][3]").unwrap(), ParamType::FixedArray(Box::new(ParamType::FixedArray(Box::new(ParamType::Bytes), 45)), 3));
	}

	#[test]
	fn test_read_tuple_param() {
		assert_eq!(Reader::read("(address,bool)").unwrap(), ParamType::Tuple(vec![ParamType::Address, ParamType::Bool]));
		assert_eq!(Reader::read("(bool[3],uint256)").unwrap(), ParamType::Tuple(vec![ParamType::FixedArray(Box::new(ParamType::Bool), 3), ParamType::Uint(256)]));
	}

	#[test]
	fn test_read_mixed_arrays() {
		assert_eq!(Reader::read("bool[][3]").unwrap(), ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 3));
		assert_eq!(Reader::read("bool[3][]").unwrap(), ParamType::Array(Box::new(ParamType::FixedArray(Box::new(ParamType::Bool), 3))));
	}
}
