// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utils used by different modules.

use crate::{Word, Error};

/// Converts a vector of bytes with len equal n * 32, to a vector of slices.
pub fn slice_data(data: &[u8]) -> Result<Vec<Word>, Error> {
	if data.len() % 32 != 0 {
		return Err(Error::InvalidData);
	}

	let times = data.len() / 32;
	let mut result = Vec::with_capacity(times);
	for i in 0..times {
		let mut slice = [0u8; 32];
		let offset = 32 * i;
		slice.copy_from_slice(&data[offset..offset + 32]);
		result.push(slice);
	}
	Ok(result)
}

/// Converts a u32 to a right aligned array of 32 bytes.
pub fn pad_u32(value: u32) -> Word {
	let mut padded = [0u8; 32];
	padded[28] = (value >> 24) as u8;
	padded[29] = (value >> 16) as u8;
	padded[30] = (value >> 8) as u8;
	padded[31] = value as u8;
	padded
}

/// Converts an i128 to a right aligned array of 32 bytes.
pub fn pad_i128(value: i128) -> Word {
	if value >= 0 {
		let mut padded = [0u8; 32];
		padded[16..].copy_from_slice(&value.to_be_bytes());
		return padded;
	}

	let mut padded = [0xffu8; 32];
	for (idx, byte) in padded.iter_mut().enumerate().skip(16) {
		*byte = (value >> 8 * (31 - idx)) as u8;
	}
	padded
}

#[cfg(test)]
mod tests {
	use hex_literal::hex;
	use super::{pad_i128, pad_u32};

	#[test]
	fn test_pad_u32() {
		// this will fail if endianness is not supported
		assert_eq!(
			pad_u32(0).to_vec(),
			hex!("0000000000000000000000000000000000000000000000000000000000000000").to_vec()
		);
		assert_eq!(
			pad_u32(1).to_vec(),
			hex!("0000000000000000000000000000000000000000000000000000000000000001").to_vec()
		);
		assert_eq!(
			pad_u32(0x100).to_vec(),
			hex!("0000000000000000000000000000000000000000000000000000000000000100").to_vec()
		);
		assert_eq!(
			pad_u32(0xffffffff).to_vec(),
			hex!("00000000000000000000000000000000000000000000000000000000ffffffff").to_vec()
		);
	}

	#[test]
	fn test_pad_i128() {
		assert_eq!(
			pad_i128(0).to_vec(),
			hex!("0000000000000000000000000000000000000000000000000000000000000000").to_vec()
		);
		assert_eq!(
			pad_i128(1).to_vec(),
			hex!("0000000000000000000000000000000000000000000000000000000000000001").to_vec()
		);
		assert_eq!(
			pad_i128(0x01000000000000000000000000000000).to_vec(),
			hex!("0000000000000000000000000000000001000000000000000000000000000000").to_vec()
		);
		assert_eq!(
			pad_i128(0xffffffff).to_vec(),
			hex!("00000000000000000000000000000000000000000000000000000000ffffffff").to_vec()
		);

		assert_eq!(
			pad_i128(-1).to_vec(),
			hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").to_vec()
		);
		assert_eq!(
			pad_i128(-2).to_vec(),
			hex!("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").to_vec()
		);
		assert_eq!(
			pad_i128(-256).to_vec(),
			hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00").to_vec()
		);
		assert_eq!(
			pad_i128(-512).to_vec(),
			hex!("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe00").to_vec()
		);
	}
}
