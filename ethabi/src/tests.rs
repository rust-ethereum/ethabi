// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{decode, encode, ParamType, Token};
#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, boxed::Box};
use hex_literal::hex;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;

pub(crate) fn assert_json_eq(left: &str, right: &str) {
	let left: Value = serde_json::from_str(left).unwrap();
	let right: Value = serde_json::from_str(right).unwrap();
	assert_eq!(left, right);
}

pub(crate) fn assert_ser_de<T>(canon: &T)
where
	T: Serialize + for<'a> Deserialize<'a> + PartialEq + Debug,
{
	let ser = serde_json::to_string(canon).unwrap();
	let de = serde_json::from_str(&ser).unwrap();
	assert_eq!(canon, &de);
}

macro_rules! test_encode_decode {
	(name: $name:tt, types: $types:expr, tokens: $tokens:expr, data: $data:tt) => {
		paste::item! {
			#[test]
			fn [<encode_ $name>]() {
				let encoded = encode(&$tokens);
				let expected = hex!($data).to_vec();
				assert_eq!(encoded, expected);
			}

			#[test]
			fn [<decode_ $name>]() {
				let encoded = hex!($data);
				let expected = $tokens;
				let decoded = decode(&$types, &encoded).unwrap();
				assert_eq!(decoded, expected);
			}
		}
	};
}

// test address
test_encode_decode! {
	name: address,
	types: [ParamType::Address],
	tokens: [Token::Address([0x11u8; 20].into())],
	data: "0000000000000000000000001111111111111111111111111111111111111111"
}
test_encode_decode! {
	name: addresses,
	types: [
	  ParamType::Address,
	  ParamType::Address
	],
	tokens: [
	  Token::Address([0x11u8; 20].into()),
	  Token::Address([0x22u8; 20].into())
	],
	data: "
		0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000002222222222222222222222222222222222222222"
}

// test bytes
test_encode_decode! {
	name: bytes,
	types: [ParamType::Bytes],
	tokens: [Token::Bytes(vec![0x12, 0x34])],
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000002
		1234000000000000000000000000000000000000000000000000000000000000"
}
test_encode_decode! {
	name: bytes2,
	types: [ParamType::Bytes],
	tokens: [Token::Bytes(hex!("10000000000000000000000000000000000000000000000000000000000002").to_vec())],
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		000000000000000000000000000000000000000000000000000000000000001f
		1000000000000000000000000000000000000000000000000000000000000200"
}
test_encode_decode! {
	name: bytes3,
	types: [ParamType::Bytes],
	tokens: [
		Token::Bytes(hex!("
			1000000000000000000000000000000000000000000000000000000000000000
			1000000000000000000000000000000000000000000000000000000000000000
		").to_vec())
	],
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000040
		1000000000000000000000000000000000000000000000000000000000000000
		1000000000000000000000000000000000000000000000000000000000000000"
}
test_encode_decode! {
	name: two_bytes,
	types: [ParamType::Bytes, ParamType::Bytes],
	tokens: [
		Token::Bytes(hex!("10000000000000000000000000000000000000000000000000000000000002").to_vec()),
		Token::Bytes(hex!("0010000000000000000000000000000000000000000000000000000000000002").to_vec())
	],
	data: "
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000080
		000000000000000000000000000000000000000000000000000000000000001f
		1000000000000000000000000000000000000000000000000000000000000200
		0000000000000000000000000000000000000000000000000000000000000020
		0010000000000000000000000000000000000000000000000000000000000002"
}

// test int
test_encode_decode! {
	name: int,
	types: [ParamType::Int(32)],
	tokens: [Token::Int([0x11u8; 32].into())],
	data: "1111111111111111111111111111111111111111111111111111111111111111"
}
test_encode_decode! {
	name: int2,
	types: [ParamType::Int(32)],
	tokens: {
		let mut int = [0u8; 32];
		int[31] = 4;
		[Token::Int(int.into())]
	},
	data: "0000000000000000000000000000000000000000000000000000000000000004"
}

// test uint
test_encode_decode! {
	name: uint,
	types: [ParamType::Uint(32)],
	tokens: [Token::Uint([0x11u8; 32].into())],
	data: "1111111111111111111111111111111111111111111111111111111111111111"
}
test_encode_decode! {
	name: uint2,
	types: [ParamType::Uint(32)],
	tokens: {
		let mut uint = [0u8; 32];
		uint[31] = 4;
		[Token::Uint(uint.into())]
	},
	data: "0000000000000000000000000000000000000000000000000000000000000004"
}

// test bool
test_encode_decode! {
	name: bool,
	types: [ParamType::Bool],
	tokens: [Token::Bool(true)],
	data: "0000000000000000000000000000000000000000000000000000000000000001"
}
test_encode_decode! {
	name: bool2,
	types: [ParamType::Bool],
	tokens: [Token::Bool(false)],
	data: "0000000000000000000000000000000000000000000000000000000000000000"
}

// test string
test_encode_decode! {
	name: string,
	types: [ParamType::String],
	tokens: [Token::String("gavofyork".to_owned())],
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000009
		6761766f66796f726b0000000000000000000000000000000000000000000000"
}

// test array
test_encode_decode! {
	name: dynamic_array_of_addresses,
	types: [ParamType::Array(Box::new(ParamType::Address))],
	tokens: {
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		[Token::Array(vec![address1, address2])]
	},
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000002222222222222222222222222222222222222222"
}
test_encode_decode! {
	name: dynamic_array_of_fixed_arrays_of_addresses,
	types: [
		ParamType::Array(Box::new(
			ParamType::FixedArray(Box::new(ParamType::Address), 2)
		))
	],
	tokens: {
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let address3 = Token::Address([0x33u8; 20].into());
		let address4 = Token::Address([0x44u8; 20].into());
		let array0 = Token::FixedArray(vec![address1, address2]);
		let array1 = Token::FixedArray(vec![address3, address4]);
		[Token::Array(vec![array0, array1])]
	},
	   data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000002222222222222222222222222222222222222222
		0000000000000000000000003333333333333333333333333333333333333333
		0000000000000000000000004444444444444444444444444444444444444444"
}
test_encode_decode! {
	name: dynamic_array_of_fixed_arrays_of_dynamic_array,
	types: [
		ParamType::Array(Box::new(
			ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Address))), 2)
		))
	],
	tokens: {
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let address3 = Token::Address([0x33u8; 20].into());
		let address4 = Token::Address([0x44u8; 20].into());
		let address5 = Token::Address([0x55u8; 20].into());
		let address6 = Token::Address([0x66u8; 20].into());
		let address7 = Token::Address([0x77u8; 20].into());
		let address8 = Token::Address([0x88u8; 20].into());
		let array0 = Token::FixedArray(vec![
			Token::Array(vec![address1, address2]),
			Token::Array(vec![address3, address4]),
		]);
		let array1 = Token::FixedArray(vec![
			Token::Array(vec![address5, address6]),
			Token::Array(vec![address7, address8]),
		]);
		[Token::Array(vec![array0, array1])]
	},
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000140
		0000000000000000000000000000000000000000000000000000000000000040
		00000000000000000000000000000000000000000000000000000000000000a0
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000002222222222222222222222222222222222222222
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000003333333333333333333333333333333333333333
		0000000000000000000000004444444444444444444444444444444444444444
		0000000000000000000000000000000000000000000000000000000000000040
		00000000000000000000000000000000000000000000000000000000000000a0
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000005555555555555555555555555555555555555555
		0000000000000000000000006666666666666666666666666666666666666666
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000007777777777777777777777777777777777777777
		0000000000000000000000008888888888888888888888888888888888888888"
	// outer array:
	//   0: 0000000000000000000000000000000000000000000000000000000000000020
	//  32: 0000000000000000000000000000000000000000000000000000000000000002 len outer => 2
	//  64: 0000000000000000000000000000000000000000000000000000000000000040 tail of outer => offset of array0
	//  96: 0000000000000000000000000000000000000000000000000000000000000140
	// array0:
	// 128: 0000000000000000000000000000000000000000000000000000000000000040 tail offset of array0 => offset of array0[0]
	// 160: 00000000000000000000000000000000000000000000000000000000000000a0 offset of array0[1] => 160
	// array0[0]:
	// 192: 0000000000000000000000000000000000000000000000000000000000000002 len of dynamic array array0[0] => 2
	// 224: 0000000000000000000000001111111111111111111111111111111111111111 array0[0][0] = address1
	// 256: 0000000000000000000000002222222222222222222222222222222222222222 array0[0][1] = address2
	// array0[1]:
	// 288: 0000000000000000000000000000000000000000000000000000000000000002 len of dynamic array0[1][0]
	// 320: 0000000000000000000000003333333333333333333333333333333333333333 array0[1][0] = address3
	// 352: 0000000000000000000000004444444444444444444444444444444444444444 array0[1][1] = address4
	// 384: 0000000000000000000000000000000000000000000000000000000000000040
	// 416: 00000000000000000000000000000000000000000000000000000000000000a0
	// 448: 0000000000000000000000000000000000000000000000000000000000000002
	// 480: 0000000000000000000000005555555555555555555555555555555555555555
	// 512: 0000000000000000000000006666666666666666666666666666666666666666
	// 544: 0000000000000000000000000000000000000000000000000000000000000002
	// 576: 0000000000000000000000007777777777777777777777777777777777777777
	// 608: 0000000000000000000000008888888888888888888888888888888888888888"

}
test_encode_decode! {
	name: dynamic_array_of_dynamic_arrays,
	types: [
		ParamType::Array(Box::new(
			ParamType::Array(Box::new(ParamType::Address))
		))
	],
	tokens: {
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let array0 = Token::Array(vec![address1]);
		let array1 = Token::Array(vec![address2]);
		let dynamic = Token::Array(vec![array0, array1]);
		[dynamic]
	},
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000080
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000002222222222222222222222222222222222222222"

	// Encoding explanation:
	// line 1 at 0x00 =   0: tail offset of dynamic array (0x20 = 32 => line 2)
	// line 2 at 0x20 =  32: length of dynamic array (0x2 = 2)
	// line 3 at 0x40 =  64: offset of array0 (0x80 = 128 = 5 * 32 => line 5)
	// line 4 at 0x60 =  96: offset of array1 (0xc0 = 192 = 7 * 32 => line 7)
	// line 5 at 0x80 = 128: length of array0 (0x1 = 1)
	// line 6 at 0xa0 = 160: value array0[0] (0x1111111111111111111111111111111111111111)
	// line 7 at 0xc0 = 192: length of array1 (0x1 = 1)
	// line 8 at 0xe0 = 224: value array1[0] (0x2222222222222222222222222222222222222222)
}
test_encode_decode! {
	name: dynamic_array_of_dynamic_arrays2,
	types: [
		ParamType::Array(Box::new(
			ParamType::Array(Box::new(ParamType::Address))
		))
	],
	tokens: {
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let address3 = Token::Address([0x33u8; 20].into());
		let address4 = Token::Address([0x44u8; 20].into());
		let array0 = Token::Array(vec![address1, address2]);
		let array1 = Token::Array(vec![address3, address4]);
		let dynamic = Token::Array(vec![array0, array1]);
		[dynamic]
	},
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000000000000000000000000000000000000000000040
		00000000000000000000000000000000000000000000000000000000000000a0
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000002222222222222222222222222222222222222222
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000003333333333333333333333333333333333333333
		0000000000000000000000004444444444444444444444444444444444444444"
}
test_encode_decode! {
	name: dynamic_array_of_bool,
	types: [ParamType::Array(Box::new(ParamType::Bool))],
	tokens: {
		[Token::Array(vec![Token::Bool(true), Token::Bool(false)])]
	},
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000000
	"
}
test_encode_decode! {
	name: dynamic_array_of_bytes,
	types: [ParamType::Array(Box::new(ParamType::Bytes))],
	tokens: {
		let bytes = hex!("019c80031b20d5e69c8093a571162299032018d913930d93ab320ae5ea44a4218a274f00d607").to_vec();
		[Token::Array(vec![Token::Bytes(bytes)])]
	},
	// line 1 at 0x00 =   0: tail offset of array
	// line 2 at 0x20 =  32: length of array
	// line 3 at 0x40 =  64: offset of array[0] (bytes)
	// line 4 at 0x60 =  96: length of array[0] (bytes)
	// line 5 at 0x80 = 128: first word of bytes
	// line 6 at 0xa0 = 160: first word of bytes
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000026
		019c80031b20d5e69c8093a571162299032018d913930d93ab320ae5ea44a421
		8a274f00d6070000000000000000000000000000000000000000000000000000"
}
test_encode_decode! {
	name: dynamic_array_of_bytes2,
	types: [ParamType::Array(Box::new(ParamType::Bytes))],
	tokens: [
		Token::Array(vec![
			Token::Bytes(hex!("4444444444444444444444444444444444444444444444444444444444444444444444444444").to_vec()),
			Token::Bytes(hex!("6666666666666666666666666666666666666666666666666666666666666666666666666666").to_vec()),
		])
	],
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000000000000000000000000000000000000000000040
		00000000000000000000000000000000000000000000000000000000000000a0
		0000000000000000000000000000000000000000000000000000000000000026
		4444444444444444444444444444444444444444444444444444444444444444
		4444444444440000000000000000000000000000000000000000000000000000
		0000000000000000000000000000000000000000000000000000000000000026
		6666666666666666666666666666666666666666666666666666666666666666
		6666666666660000000000000000000000000000000000000000000000000000"
}
test_encode_decode! {
	name: empty_dynamic_array,
	types: [
		ParamType::Array(Box::new(ParamType::Bool)),
		ParamType::Array(Box::new(ParamType::Bool)),
	],
	tokens: [
		Token::Array(vec![]),
		Token::Array(vec![])
	],
	data: "
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000060
		0000000000000000000000000000000000000000000000000000000000000000
		0000000000000000000000000000000000000000000000000000000000000000"
}
test_encode_decode! {
	name: dynamic_array_of_empty_dynamic_array,
	types: [
		ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Bool)))),
		ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Bool)))),
	],
	tokens: [
		Token::Array(vec![Token::Array(vec![])]),
		Token::Array(vec![Token::Array(vec![])]),
	],
	data: "
		0000000000000000000000000000000000000000000000000000000000000040
		00000000000000000000000000000000000000000000000000000000000000a0
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000000
		0000000000000000000000000000000000000000000000000000000000000001
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000000"
}

// test fixed array
test_encode_decode! {
	name: fixed_array_of_addresses,
	types: [ParamType::FixedArray(Box::new(ParamType::Address), 2)],
	tokens: {
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		[Token::FixedArray(vec![address1, address2])]
	},
	data: "
		0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000002222222222222222222222222222222222222222"
}
test_encode_decode! {
	name: fixed_array_of_strings,
	types: [ParamType::FixedArray(Box::new(ParamType::String), 2)],
	tokens: {
		let s1 = Token::String("foo".into());
		let s2 = Token::String("bar".into());
		[Token::FixedArray(vec![s1, s2])]
	},
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000040
		0000000000000000000000000000000000000000000000000000000000000080
		0000000000000000000000000000000000000000000000000000000000000003
		666f6f0000000000000000000000000000000000000000000000000000000000
		0000000000000000000000000000000000000000000000000000000000000003
		6261720000000000000000000000000000000000000000000000000000000000"
	// `data` explained:
	// line 1 at 0x00 =   0: tail offset for the array
	// line 2 at 0x20 =  32: offset of string 1
	// line 3 at 0x40 =  64: offset of string 2
	// line 4 at 0x60 =  96: length of string 1
	// line 5 at 0x80 = 128: value  of string 1
	// line 6 at 0xa0 = 160: length of string 2
	// line 7 at 0xc0 = 192: value  of string 2
}
test_encode_decode! {
	name: fixed_array_of_fixed_arrays,
	types: [
		ParamType::FixedArray(
			Box::new(ParamType::FixedArray(Box::new(ParamType::Address), 2)),
			2
		)
	],
	tokens: {
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let address3 = Token::Address([0x33u8; 20].into());
		let address4 = Token::Address([0x44u8; 20].into());
		let array0 = Token::FixedArray(vec![address1, address2]);
		let array1 = Token::FixedArray(vec![address3, address4]);
		let fixed = Token::FixedArray(vec![array0, array1]);
		[fixed]
	},
	data: "
		0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000002222222222222222222222222222222222222222
		0000000000000000000000003333333333333333333333333333333333333333
		0000000000000000000000004444444444444444444444444444444444444444"
}
test_encode_decode! {
	name: fixed_array_of_dynamic_array_of_addresses,
	types: [
		ParamType::FixedArray(
			Box::new(ParamType::Array(Box::new(ParamType::Address))),
			2
		)
	],
	tokens: {
		let address1 = Token::Address([0x11u8; 20].into());
		let address2 = Token::Address([0x22u8; 20].into());
		let address3 = Token::Address([0x33u8; 20].into());
		let address4 = Token::Address([0x44u8; 20].into());
		let array0 = Token::Array(vec![address1, address2]);
		let array1 = Token::Array(vec![address3, address4]);
		[Token::FixedArray(vec![array0, array1])]
	},
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000040
		00000000000000000000000000000000000000000000000000000000000000a0
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000001111111111111111111111111111111111111111
		0000000000000000000000002222222222222222222222222222222222222222
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000003333333333333333333333333333333333333333
		0000000000000000000000004444444444444444444444444444444444444444"
}

// test fixed bytes
test_encode_decode! {
	name: fixed_bytes,
	types: [ParamType::FixedBytes(2)],
	tokens: [Token::FixedBytes(vec![0x12, 0x34])],
	data: "1234000000000000000000000000000000000000000000000000000000000000"

}

// test tuple with tuple array member
test_encode_decode! {
	name: tuple_with_tuple_array_test,
	types: [
		ParamType::Tuple(vec![
			ParamType::Array(Box::new(ParamType::Tuple(
				vec![
					ParamType::Address,
					ParamType::Uint(256)
				]
			)))
		])
	],
	tokens: {
		[
			Token::Tuple(
				vec![
					Token::Array(vec![
						Token::Tuple(vec![
							Token::Address([0x11u8; 20].into()),
							Token::Uint([0x11u8; 32].into()),
						]),
						Token::Tuple(vec![
							Token::Address([0x22u8; 20].into()),
							Token::Uint([0x22u8; 32].into()),
						]),
						Token::Tuple(vec![
							Token::Address([0x33u8; 20].into()),
							Token::Uint([0x44u8; 32].into()),
						])
					])
				]
			)
		]
	},
	data: "
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000020
		0000000000000000000000000000000000000000000000000000000000000003
		0000000000000000000000001111111111111111111111111111111111111111
		1111111111111111111111111111111111111111111111111111111111111111
		0000000000000000000000002222222222222222222222222222222222222222
		2222222222222222222222222222222222222222222222222222222222222222
		0000000000000000000000003333333333333333333333333333333333333333
		4444444444444444444444444444444444444444444444444444444444444444
	"
}

// comprehensive test
test_encode_decode! {
	name: comprehensive_test,
	types: [
		ParamType::Int(32),
		ParamType::Bytes,
		ParamType::Int(32),
		ParamType::Bytes,
	],
	tokens: {
		let bytes = hex!("
			131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
			131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
		").to_vec();
		[
			Token::Int(5.into()),
			Token::Bytes(bytes.clone()),
			Token::Int(3.into()),
			Token::Bytes(bytes),
		]
	},
	data: "
		0000000000000000000000000000000000000000000000000000000000000005
		0000000000000000000000000000000000000000000000000000000000000080
		0000000000000000000000000000000000000000000000000000000000000003
		00000000000000000000000000000000000000000000000000000000000000e0
		0000000000000000000000000000000000000000000000000000000000000040
		131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
		131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
		0000000000000000000000000000000000000000000000000000000000000040
		131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b
		131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b"
}
test_encode_decode! {
	name: comprehensive_test2,
	types: [
		ParamType::Int(32),
		ParamType::String,
		ParamType::Int(32),
		ParamType::Int(32),
		ParamType::Int(32),
		ParamType::Array(Box::new(ParamType::Int(32))),
	],
	tokens: [
		Token::Int(1.into()),
		Token::String("gavofyork".to_owned()),
		Token::Int(2.into()),
		Token::Int(3.into()),
		Token::Int(4.into()),
		Token::Array(vec![
			Token::Int(5.into()),
			Token::Int(6.into()),
			Token::Int(7.into()),
		])
	],
	data: "
		0000000000000000000000000000000000000000000000000000000000000001
		00000000000000000000000000000000000000000000000000000000000000c0
		0000000000000000000000000000000000000000000000000000000000000002
		0000000000000000000000000000000000000000000000000000000000000003
		0000000000000000000000000000000000000000000000000000000000000004
		0000000000000000000000000000000000000000000000000000000000000100
		0000000000000000000000000000000000000000000000000000000000000009
		6761766f66796f726b0000000000000000000000000000000000000000000000
		0000000000000000000000000000000000000000000000000000000000000003
		0000000000000000000000000000000000000000000000000000000000000005
		0000000000000000000000000000000000000000000000000000000000000006
		0000000000000000000000000000000000000000000000000000000000000007"
}
