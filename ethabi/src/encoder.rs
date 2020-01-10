//! ABI encoder.

use crate::util::pad_u32;
use crate::{Word, Token, Bytes};

fn pad_bytes(bytes: &[u8]) -> Vec<Word> {
	let mut result = vec![pad_u32(bytes.len() as u32)];
	result.extend(pad_fixed_bytes(bytes));
	result
}

fn pad_fixed_bytes(bytes: &[u8]) -> Vec<Word> {
	let len = (bytes.len() + 31) / 32;
	let mut result = Vec::with_capacity(len);
	for i in 0..len {
		let mut padded = [0u8; 32];

		let to_copy = match i == len - 1 {
			false => 32,
			true => match bytes.len() % 32 {
				0 => 32,
				x => x,
			},
		};

		let offset = 32 * i;
		padded[..to_copy].copy_from_slice(&bytes[offset..offset + to_copy]);
		result.push(padded);
	}

	result
}

#[derive(Debug)]
enum Mediate {
	Raw(Vec<Word>),
	Prefixed(Vec<Word>),
	PrefixedArray(Vec<Mediate>),
	PrefixedArrayWithLength(Vec<Mediate>),
}

impl Mediate {
	fn head_len(&self) -> u32 {
		match *self {
			Mediate::Raw(ref raw) => 32 * raw.len() as u32,
			Mediate::Prefixed(_) | Mediate::PrefixedArray(_) | Mediate::PrefixedArrayWithLength(_) => 32,
		}
	}

	fn tail_len(&self) -> u32 {
		match *self {
			Mediate::Raw(_) => 0,
			Mediate::Prefixed(ref pre) => pre.len() as u32 * 32,
			Mediate::PrefixedArray(ref mediates) => mediates.iter().fold(0, |acc, m| acc + m.head_len() + m.tail_len()),
			Mediate::PrefixedArrayWithLength(ref mediates) => mediates.iter().fold(32, |acc, m| acc + m.head_len() + m.tail_len()),
		}
	}

	fn head(&self, suffix_offset: u32) -> Vec<Word> {
		match *self {
			Mediate::Raw(ref raw) => raw.clone(),
			Mediate::Prefixed(_) | Mediate::PrefixedArray(_) | Mediate::PrefixedArrayWithLength(_) => {
				vec![pad_u32(suffix_offset)]
			}
		}
	}

	fn tail(&self) -> Vec<Word> {
		match *self {
			Mediate::Raw(_) => vec![],
			Mediate::Prefixed(ref raw) => raw.clone(),
			Mediate::PrefixedArray(ref mediates) => encode_head_tail(mediates),
			Mediate::PrefixedArrayWithLength(ref mediates) => {
				// + 32 added to offset represents len of the array prepanded to tail
				let mut result = vec![pad_u32(mediates.len() as u32)];

				let head_tail = encode_head_tail(mediates);

				result.extend(head_tail);
				result
			},
		}
	}
}

fn encode_head_tail(mediates: &Vec<Mediate>) -> Vec<Word> {
	let heads_len = mediates.iter()
		.fold(0, |acc, m| acc + m.head_len());

	let (mut result, len) = mediates.iter()
		.fold(
			(Vec::with_capacity(heads_len as usize), heads_len),
			|(mut acc, offset), m| {
				acc.extend(m.head(offset));
				(acc, offset + m.tail_len())
			}
		);

	let tails = mediates.iter()
		.fold(
			Vec::with_capacity((len - heads_len) as usize),
			|mut acc, m| {
				acc.extend(m.tail());
				acc
			}
		);

	result.extend(tails);
	result
}

/// Encodes vector of tokens into ABI compliant vector of bytes.
pub fn encode(tokens: &[Token]) -> Bytes {
	let mediates = &tokens.iter()
		.map(encode_token)
		.collect();

	encode_head_tail(mediates).iter()
		.flat_map(|word| word.to_vec())
		.collect()
}

fn encode_token(token: &Token) -> Mediate {
	match *token {
		Token::Address(ref address) => {
			let mut padded = [0u8; 32];
			padded[12..].copy_from_slice(address.as_ref());
			Mediate::Raw(vec![padded])
		},
		Token::Bytes(ref bytes) => Mediate::Prefixed(pad_bytes(bytes)),
		Token::String(ref s) => Mediate::Prefixed(pad_bytes(s.as_bytes())),
		Token::FixedBytes(ref bytes) => Mediate::Raw(pad_fixed_bytes(bytes)),
		Token::Int(int) => Mediate::Raw(vec![int.into()]),
		Token::Uint(uint) => Mediate::Raw(vec![uint.into()]),
		Token::Bool(b) => {
			let mut value = [0u8; 32];
			if b {
				value[31] = 1;
			}
			Mediate::Raw(vec![value])
		},
		Token::Array(ref tokens) => {
			let mediates = tokens.iter()
				.map(encode_token)
				.collect();

			Mediate::PrefixedArrayWithLength(mediates)
		},
		Token::FixedArray(ref tokens) => {
			let mediates = tokens.iter()
				.map(encode_token)
				.collect();

			if token.is_dynamic() {
				Mediate::PrefixedArray(mediates)
			} else {
				Mediate::Raw(encode_head_tail(&mediates))
			}
		},
	}
}

