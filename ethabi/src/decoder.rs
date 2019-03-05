//! ABI decoder.

use {Error, ErrorKind, ParamType, ResultExt, Token};

#[derive(Debug)]
struct DecodeResult {
	token: Token,
	new_offset: usize,
}

fn as_usize(slice: &[u8; 32]) -> Result<usize, Error> {
	if !slice[..28].iter().all(|x| *x == 0) {
		return Err(ErrorKind::InvalidData.into());
	}

	let result = ((slice[28] as usize) << 24)
		+ ((slice[29] as usize) << 16)
		+ ((slice[30] as usize) << 8)
		+ (slice[31] as usize);

	Ok(result)
}

fn as_bool(slice: &[u8; 32]) -> Result<bool, Error> {
	if !slice[..31].iter().all(|x| *x == 0) {
		return Err(ErrorKind::InvalidData.into());
	}

	Ok(slice[31] == 1)
}

/// Decodes ABI compliant vector of bytes into vector of tokens described by types param.
pub fn decode(types: &[ParamType], data: &[u8]) -> Result<Vec<Token>, Error> {
	let is_empty_bytes_valid_encoding = types.iter().all(|t| t.is_empty_bytes_valid_encoding());
	if !is_empty_bytes_valid_encoding && data.is_empty() {
		bail!(
			"please ensure the contract and method you're calling exist! \
			 failed to decode empty bytes. \
			 if you're using jsonrpc this is likely due to jsonrpc returning \
			 `0x` in case contract or method don't exist"
		);
	}

	let mut tokens = vec![];
	let mut offset = 0;

	for param in types {
		let res =
			decode_param(param, data, offset).chain_err(|| format!("Cannot decode {}", param))?;
		offset = res.new_offset;
		tokens.push(res.token);
	}

	Ok(tokens)
}

fn peek(data: &[u8], offset: usize, len: usize) -> Result<&[u8], Error> {
	if offset + len > data.len() {
		return Err(ErrorKind::InvalidData.into());
	} else {
		Ok(&data[offset..(offset + len)])
	}
}

fn peek_32_bytes(data: &[u8], offset: usize) -> Result<[u8; 32], Error> {
	peek(data, offset, 32).map(|x| {
		let mut out: [u8; 32] = [0u8; 32];
		out.copy_from_slice(&x[0..32]);
		out
	})
}

fn take_bytes(data: &[u8], offset: usize, len: usize) -> Result<Vec<u8>, Error> {
	if offset + len > data.len() {
		return Err(ErrorKind::InvalidData.into());
	} else {
		Ok((&data[offset..(offset + len)]).to_vec())
	}
}

fn decode_param(param: &ParamType, data: &[u8], offset: usize) -> Result<DecodeResult, Error> {
	match *param {
		ParamType::Address => {
			let slice = peek_32_bytes(data, offset)?;
			let mut address = [0u8; 20];
			address.copy_from_slice(&slice[12..]);
			let result = DecodeResult {
				token: Token::Address(address.into()),
				new_offset: offset + 32,
			};
			Ok(result)
		}
		ParamType::Int(_) => {
			let slice = peek_32_bytes(data, offset)?;
			let result = DecodeResult {
				token: Token::Int(slice.clone().into()),
				new_offset: offset + 32,
			};
			Ok(result)
		}
		ParamType::Uint(_) => {
			let slice = peek_32_bytes(data, offset)?;
			let result = DecodeResult {
				token: Token::Uint(slice.clone().into()),
				new_offset: offset + 32,
			};
			Ok(result)
		}
		ParamType::Bool => {
			let b = as_bool(&peek_32_bytes(data, offset)?)?;
			let result = DecodeResult {
				token: Token::Bool(b),
				new_offset: offset + 32,
			};
			Ok(result)
		}
		ParamType::FixedBytes(len) => {
			let bytes = take_bytes(data, offset, len)?;
			let result = DecodeResult {
				token: Token::FixedBytes(bytes),
				new_offset: offset + 32,
			};
			Ok(result)
		}
		ParamType::Bytes => {
			let dynamic_offset = as_usize(&peek_32_bytes(data, offset)?)?;
			let len = as_usize(&peek_32_bytes(data, dynamic_offset)?)?;
			let bytes = take_bytes(data, dynamic_offset + 32, len)?;
			let result = DecodeResult {
				token: Token::Bytes(bytes),
				new_offset: offset + 32,
			};
			Ok(result)
		}
		ParamType::String => {
			let dynamic_offset = as_usize(&peek_32_bytes(data, offset)?)?;
			let len = as_usize(&peek_32_bytes(data, dynamic_offset)?)?;
			let bytes = take_bytes(data, dynamic_offset + 32, len)?;
			let result = DecodeResult {
				token: Token::String(String::from_utf8(bytes)?),
				new_offset: offset + 32,
			};
			Ok(result)
		}
		ParamType::Array(ref t) => {
			let dynamic_offset = as_usize(&peek_32_bytes(data, offset)?)?;
			let len = as_usize(&peek_32_bytes(data, dynamic_offset)?)?;
			let mut tokens = vec![];
			let mut new_offset = dynamic_offset + 32;

			for _ in 0..len {
				let res = decode_param(t, data, new_offset)?;
				new_offset = res.new_offset;
				tokens.push(res.token);
			}

			let result = DecodeResult {
				token: Token::Array(tokens),
				new_offset: offset + 32,
			};

			Ok(result)
		}
		ParamType::FixedArray(ref t, len) => {
			let mut tokens = vec![];
			let mut new_offset = offset;

			for _ in 0..len {
				let res = decode_param(t, data, new_offset)?;
				new_offset = res.new_offset;
				tokens.push(res.token);
			}

			let result = DecodeResult {
				token: Token::FixedArray(tokens),
				new_offset,
			};

			Ok(result)
		}
	}
}

#[cfg(test)]
mod tests {
	use {decode, ParamType, Token, Uint};

	#[test]
	fn decode_address() {
		let encoded = hex!("0000000000000000000000001111111111111111111111111111111111111111");
		let address = Token::Address([0x11u8; 20].into());
		let expected = vec![address];
		let decoded = decode(&[ParamType::Address], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_two_address() {
		let encoded = hex!("
					   0000000000000000000000001111111111111111111111111111111111111111
					   0000000000000000000000002222222222222222222222222222222222222222
					");
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let expected = vec![address1, address2];
		let decoded = decode(&[ParamType::Address, ParamType::Address], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_fixed_array_of_addresses() {
		let encoded = hex!("
					   0000000000000000000000001111111111111111111111111111111111111111
					   0000000000000000000000002222222222222222222222222222222222222222
					");
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let expected = vec![Token::FixedArray(vec![address1, address2])];
		let decoded = decode(&[ParamType::FixedArray(Box::new(ParamType::Address), 2)], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_uint() {
		let encoded = hex!("1111111111111111111111111111111111111111111111111111111111111111");
		let uint = Token::Uint([0x11u8; 32].into());
		let expected = vec![uint];
		let decoded = decode(&[ParamType::Uint(32)], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_int() {
		let encoded = hex!("1111111111111111111111111111111111111111111111111111111111111111");
		let int = Token::Int([0x11u8; 32].into());
		let expected = vec![int];
		let decoded = decode(&[ParamType::Int(32)], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_dynamic_array_of_addresses() {
		let encoded = hex!("
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000002
			0000000000000000000000001111111111111111111111111111111111111111
			0000000000000000000000002222222222222222222222222222222222222222
		");
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let addresses = Token::Array(vec![address1, address2]);
		let expected = vec![addresses];
		let decoded = decode(&[ParamType::Array(Box::new(ParamType::Address))], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_dynamic_array_of_fixed_arrays() {
		let encoded = hex!("
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000002
			0000000000000000000000001111111111111111111111111111111111111111
			0000000000000000000000002222222222222222222222222222222222222222
			0000000000000000000000003333333333333333333333333333333333333333
			0000000000000000000000004444444444444444444444444444444444444444
		");
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let address3 = Token::Address([0x33u8; 20].into());
		let address4 = Token::Address([0x44u8; 20].into());
		let array0 = Token::FixedArray(vec![address1, address2]);
		let array1 = Token::FixedArray(vec![address3, address4]);
		let dynamic = Token::Array(vec![array0, array1]);
		let expected = vec![dynamic];
		let decoded = decode(&[
			ParamType::Array(Box::new(
				ParamType::FixedArray(Box::new(ParamType::Address), 2)
			))
		], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_dynamic_array_of_dynamic_arrays() {
		let encoded  = hex!("
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000002
			0000000000000000000000000000000000000000000000000000000000000080
			00000000000000000000000000000000000000000000000000000000000000c0
			0000000000000000000000000000000000000000000000000000000000000001
			0000000000000000000000001111111111111111111111111111111111111111
			0000000000000000000000000000000000000000000000000000000000000001
			0000000000000000000000002222222222222222222222222222222222222222
		");

		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let array0 = Token::Array(vec![address1]);
		let array1 = Token::Array(vec![address2]);
		let dynamic = Token::Array(vec![array0, array1]);
		let expected = vec![dynamic];
		let decoded = decode(&[
			ParamType::Array(Box::new(
				ParamType::Array(Box::new(ParamType::Address))
			))
		], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_dynamic_array_of_dynamic_arrays2() {
		let encoded = hex!("
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000002
			0000000000000000000000000000000000000000000000000000000000000080
			00000000000000000000000000000000000000000000000000000000000000e0
			0000000000000000000000000000000000000000000000000000000000000002
			0000000000000000000000001111111111111111111111111111111111111111
			0000000000000000000000002222222222222222222222222222222222222222
			0000000000000000000000000000000000000000000000000000000000000002
			0000000000000000000000003333333333333333333333333333333333333333
			0000000000000000000000004444444444444444444444444444444444444444
		");

		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let address3 = Token::Address([0x33u8; 20].into());
		let address4 = Token::Address([0x44u8; 20].into());
		let array0 = Token::Array(vec![address1, address2]);
		let array1 = Token::Array(vec![address3, address4]);
		let dynamic = Token::Array(vec![array0, array1]);
		let expected = vec![dynamic];
		let decoded = decode(&[
			ParamType::Array(Box::new(
				ParamType::Array(Box::new(ParamType::Address))
			))
		], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_fixed_array_fixed_arrays() {
		let encoded = hex!("
			0000000000000000000000001111111111111111111111111111111111111111
			0000000000000000000000002222222222222222222222222222222222222222
			0000000000000000000000003333333333333333333333333333333333333333
			0000000000000000000000004444444444444444444444444444444444444444
		");
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let address3 = Token::Address([0x33u8; 20].into());
		let address4 = Token::Address([0x44u8; 20].into());
		let array0 = Token::FixedArray(vec![address1, address2]);
		let array1 = Token::FixedArray(vec![address3, address4]);
		let fixed = Token::FixedArray(vec![array0, array1]);
		let expected = vec![fixed];

		let decoded = decode(&[
			ParamType::FixedArray(
				Box::new(ParamType::FixedArray(Box::new(ParamType::Address), 2)),
				2
			)
		], &encoded).unwrap();

		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_fixed_array_of_dynamic_array_of_addresses() {
		let encoded = hex!("
			0000000000000000000000000000000000000000000000000000000000000040
			00000000000000000000000000000000000000000000000000000000000000a0
			0000000000000000000000000000000000000000000000000000000000000002
			0000000000000000000000001111111111111111111111111111111111111111
			0000000000000000000000002222222222222222222222222222222222222222
			0000000000000000000000000000000000000000000000000000000000000002
			0000000000000000000000003333333333333333333333333333333333333333
			0000000000000000000000004444444444444444444444444444444444444444
		");
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let address3 = Token::Address([0x33u8; 20].into());
		let address4 = Token::Address([0x44u8; 20].into());
		let array0 = Token::Array(vec![address1, address2]);
		let array1 = Token::Array(vec![address3, address4]);
		let fixed = Token::FixedArray(vec![array0, array1]);
		let expected = vec![fixed];

		let decoded = decode(&[
			ParamType::FixedArray(
				Box::new(ParamType::Array(Box::new(ParamType::Address))),
				2
			)
		], &encoded).unwrap();

		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_fixed_bytes() {
		let encoded = hex!("1234000000000000000000000000000000000000000000000000000000000000");
		let bytes = Token::FixedBytes(vec![0x12, 0x34]);
		let expected = vec![bytes];
		let decoded = decode(&[ParamType::FixedBytes(2)], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_bytes() {
		let encoded = hex!("
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000002
			1234000000000000000000000000000000000000000000000000000000000000
		");
		let bytes = Token::Bytes(vec![0x12, 0x34]);
		let expected = vec![bytes];
		let decoded = decode(&[ParamType::Bytes], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_bytes2() {
		let encoded = hex!("
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000040
			1000000000000000000000000000000000000000000000000000000000000000
			1000000000000000000000000000000000000000000000000000000000000000
		");
		let bytes = Token::Bytes(hex!("
			1000000000000000000000000000000000000000000000000000000000000000
			1000000000000000000000000000000000000000000000000000000000000000
		").to_vec());
		let expected = vec![bytes];
		let decoded = decode(&[ParamType::Bytes], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_two_bytes() {
		let encoded = hex!("
			0000000000000000000000000000000000000000000000000000000000000040
			0000000000000000000000000000000000000000000000000000000000000080
			000000000000000000000000000000000000000000000000000000000000001f
			1000000000000000000000000000000000000000000000000000000000000200
			0000000000000000000000000000000000000000000000000000000000000020
			0010000000000000000000000000000000000000000000000000000000000002
		");
		let bytes1 = Token::Bytes(hex!("10000000000000000000000000000000000000000000000000000000000002").to_vec());
		let bytes2 = Token::Bytes(hex!("0010000000000000000000000000000000000000000000000000000000000002").to_vec());
		let expected = vec![bytes1, bytes2];
		let decoded = decode(&[ParamType::Bytes, ParamType::Bytes], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_string() {
		let encoded = hex!("
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000009
			6761766f66796f726b0000000000000000000000000000000000000000000000
		");
		let s = Token::String("gavofyork".to_owned());
		let expected = vec![s];
		let decoded = decode(&[ParamType::String], &encoded).unwrap();
		assert_eq!(decoded, expected);
	}

	#[test]
	fn decode_from_empty_byte_slice() {
        // these can NOT be decoded from empty byte slice
        assert!(decode(&[ParamType::Address], &[]).is_err());
        assert!(decode(&[ParamType::Bytes], &[]).is_err());
        assert!(decode(&[ParamType::Int(0)], &[]).is_err());
        assert!(decode(&[ParamType::Int(1)], &[]).is_err());
        assert!(decode(&[ParamType::Int(0)], &[]).is_err());
        assert!(decode(&[ParamType::Int(1)], &[]).is_err());
        assert!(decode(&[ParamType::Bool], &[]).is_err());
        assert!(decode(&[ParamType::String], &[]).is_err());
        assert!(decode(&[ParamType::Array(Box::new(ParamType::Bool))], &[]).is_err());
        assert!(decode(&[ParamType::FixedBytes(1)], &[]).is_err());
        assert!(decode(&[ParamType::FixedArray(Box::new(ParamType::Bool), 1)], &[]).is_err());

        // these are the only ones that can be decoded from empty byte slice
        assert!(decode(&[ParamType::FixedBytes(0)], &[]).is_ok());
        assert!(decode(&[ParamType::FixedArray(Box::new(ParamType::Bool), 0)], &[]).is_ok());
	}

	#[test]
	fn decode_data_with_size_that_is_not_a_multiple_of_32() {
		let encoded = hex!(
			"
            0000000000000000000000000000000000000000000000000000000000000000
            00000000000000000000000000000000000000000000000000000000000000a0
            0000000000000000000000000000000000000000000000000000000000000152
            0000000000000000000000000000000000000000000000000000000000000001
            000000000000000000000000000000000000000000000000000000000054840d
            0000000000000000000000000000000000000000000000000000000000000092
            3132323033393637623533326130633134633938306235616566666231373034
            3862646661656632633239336139353039663038656233633662306635663866
            3039343265376239636337366361353163636132366365353436393230343438
            6533303866646136383730623565326165313261323430396439343264653432
            3831313350373230703330667073313678390000000000000000000000000000
            0000000000000000000000000000000000103933633731376537633061363531
            3761
        "
		);

		assert_eq!(
			decode(
				&[
					ParamType::Uint(256),
					ParamType::String,
					ParamType::String,
					ParamType::Uint(256),
					ParamType::Uint(256),
				],
				&encoded,
			).unwrap(),
			&[
				Token::Uint(Uint::from(0)),
				Token::String(String::from("12203967b532a0c14c980b5aeffb17048bdfaef2c293a9509f08eb3c6b0f5f8f0942e7b9cc76ca51cca26ce546920448e308fda6870b5e2ae12a2409d942de428113P720p30fps16x9")),
				Token::String(String::from("93c717e7c0a6517a")),
				Token::Uint(Uint::from(1)),
				Token::Uint(Uint::from(5538829))
			]
		);
	}

	#[test]
	fn decode_after_fixed_bytes_with_less_than_32_bytes() {
		let encoded = hex!("
			0000000000000000000000008497afefdc5ac170a664a231f6efb25526ef813f
			0000000000000000000000000000000000000000000000000000000000000000
			0000000000000000000000000000000000000000000000000000000000000000
			0000000000000000000000000000000000000000000000000000000000000080
			000000000000000000000000000000000000000000000000000000000000000a
			3078303030303030314600000000000000000000000000000000000000000000
		");

		assert_eq!(
			decode(
				&[
					ParamType::Address,
					ParamType::FixedBytes(32),
					ParamType::FixedBytes(4),
					ParamType::String,
				],
				&encoded,
			)
			.unwrap(),
			&[
				Token::Address(hex!("8497afefdc5ac170a664a231f6efb25526ef813f").into()),
				Token::FixedBytes([0u8; 32].to_vec()),
				Token::FixedBytes([0u8; 4].to_vec()),
				Token::String("0x0000001F".into()),
			]
		);
	}
}
