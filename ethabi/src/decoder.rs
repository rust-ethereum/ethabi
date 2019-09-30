//! ABI decoder.

use util::slice_data;
use {Word, Token, ErrorKind, Error, ResultExt, ParamType};

struct DecodeResult {
	token: Token,
	new_offset: usize,
}

struct BytesTaken {
	bytes: Vec<u8>,
	new_offset: usize,
}

fn as_u32(slice: &Word) -> Result<u32, Error> {
	if !slice[..28].iter().all(|x| *x == 0) {
		return Err(ErrorKind::InvalidData.into());
	}

	let result = ((slice[28] as u32) << 24) +
		((slice[29] as u32) << 16) +
		((slice[30] as u32) << 8) +
		(slice[31] as u32);

	Ok(result)
}

fn as_bool(slice: &Word) -> Result<bool, Error> {
	if !slice[..31].iter().all(|x| *x == 0) {
		return Err(ErrorKind::InvalidData.into());
	}

	Ok(slice[31] == 1)
}

/// Decodes ABI compliant vector of bytes into vector of tokens described by types param.
pub fn decode(types: &[ParamType], data: &[u8]) -> Result<Vec<Token>, Error> {
    let is_empty_bytes_valid_encoding = types.iter().all(|t| t.is_empty_bytes_valid_encoding());
    if !is_empty_bytes_valid_encoding && data.is_empty() {
        bail!("please ensure the contract and method you're calling exist! failed to decode empty bytes. if you're using jsonrpc this is likely due to jsonrpc returning `0x` in case contract or method don't exist");
    }
	let slices = slice_data(data)?;
	let mut tokens = Vec::with_capacity(types.len());
	let mut offset = 0;
	for param in types {
		let res = decode_param(param, &slices, offset).chain_err(|| format!("Cannot decode {}", param))?;
		offset = res.new_offset;
		tokens.push(res.token);
	}
	Ok(tokens)
}

fn peek(slices: &[Word], position: usize) -> Result<&Word, Error> {
	slices.get(position).ok_or_else(|| ErrorKind::InvalidData.into())
}

fn take_bytes(slices: &[Word], position: usize, len: usize) -> Result<BytesTaken, Error> {
	let slices_len = (len + 31) / 32;

	let mut bytes_slices = Vec::with_capacity(slices_len);
	for i in 0..slices_len {
		let slice = peek(slices, position + i)?;
		bytes_slices.push(slice);
	}

	let bytes = bytes_slices.into_iter()
		.flat_map(|slice| slice.to_vec())
		.take(len)
		.collect();

	let taken = BytesTaken {
		bytes,
		new_offset: position + slices_len,
	};

	Ok(taken)
}

fn decode_param(param: &ParamType, slices: &[Word], offset: usize) -> Result<DecodeResult, Error> {
	match *param {
		ParamType::Address => {
			let slice = peek(slices, offset)?;
			let mut address = [0u8; 20];
			address.copy_from_slice(&slice[12..]);

			let result = DecodeResult {
				token: Token::Address(address.into()),
				new_offset: offset + 1,
			};

			Ok(result)
		},
		ParamType::Int(_) => {
			let slice = peek(slices, offset)?;

			let result = DecodeResult {
				token: Token::Int(slice.clone().into()),
				new_offset: offset + 1,
			};

			Ok(result)
		},
		ParamType::Uint(_) => {
			let slice = peek(slices, offset)?;

			let result = DecodeResult {
				token: Token::Uint(slice.clone().into()),
				new_offset: offset + 1,
			};

			Ok(result)
		},
		ParamType::Bool => {
			let slice = peek(slices, offset)?;

			let b = as_bool(slice)?;

			let result = DecodeResult {
				token: Token::Bool(b),
				new_offset: offset + 1,
			};

			Ok(result)
		},
		ParamType::FixedBytes(len) => {
			let taken = take_bytes(slices, offset, len)?;

			let result = DecodeResult {
				token: Token::FixedBytes(taken.bytes),
				new_offset: taken.new_offset,
			};

			Ok(result)
		},
		ParamType::Bytes => {
			let offset_slice = peek(slices, offset)?;
			let len_offset = (as_u32(offset_slice)? / 32) as usize;

			let len_slice = peek(slices, len_offset)?;
			let len = as_u32(len_slice)? as usize;

			let taken = take_bytes(slices, len_offset + 1, len)?;

			let result = DecodeResult {
				token: Token::Bytes(taken.bytes),
				new_offset: offset + 1,
			};

			Ok(result)
		},
		ParamType::String => {
			let offset_slice = peek(slices, offset)?;
			let len_offset = (as_u32(offset_slice)? / 32) as usize;

			let len_slice = peek(slices, len_offset)?;
			let len = as_u32(len_slice)? as usize;

			let taken = take_bytes(slices, len_offset + 1, len)?;

			let result = DecodeResult {
				token: Token::String(String::from_utf8(taken.bytes)?),
				new_offset: offset + 1,
			};

			Ok(result)
		},
		ParamType::Array(ref t) => {
			let offset_slice = peek(slices, offset)?;
			let len_offset = (as_u32(offset_slice)? / 32) as usize;

			let len_slice = peek(slices, len_offset)?;
			let len = as_u32(len_slice)? as usize;

			let sub_slices = &slices[len_offset + 1..];
			let mut tokens = Vec::with_capacity(len);
			let mut new_offset = 0;
			for _ in 0..len {
				let res = decode_param(t, &sub_slices, new_offset)?;
				new_offset = res.new_offset;
				tokens.push(res.token);
			}

			let result = DecodeResult {
				token: Token::Array(tokens),
				new_offset: offset + 1,
			};

			Ok(result)
		},
		ParamType::FixedArray(ref t, len) => {
			let mut tokens = Vec::with_capacity(len);
			let new_offset = if param.is_dynamic() {
				let offset_slice = peek(slices, offset)?;
				let tail_offset = (as_u32(offset_slice)? / 32) as usize;
				let slices = &slices[tail_offset..];
				let mut new_offset = 0;

				for _ in 0..len {
					let res = decode_param(t, &slices, new_offset)?;
					new_offset = res.new_offset;
					tokens.push(res.token);
				}
				offset + 1
			} else {
				let mut new_offset = offset;

				for _ in 0..len {
					let res = decode_param(t, &slices, new_offset)?;
					new_offset = res.new_offset;
					tokens.push(res.token);
				}
				new_offset
			};

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
	use {decode, ParamType};

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
}

