use errors::Error;
use token::{StrictTokenizer, Tokenizer};
use util::pad_i128;
use Uint;

/// Tries to parse string as a token. Does not require string to clearly represent the value.
pub struct LenientTokenizer;

impl Tokenizer for LenientTokenizer {
	fn tokenize_address(value: &str) -> Result<[u8; 20], Error> {
		StrictTokenizer::tokenize_address(value)
	}

	fn tokenize_string(value: &str) -> Result<String, Error> {
		StrictTokenizer::tokenize_string(value)
	}

	fn tokenize_bool(value: &str) -> Result<bool, Error> {
		StrictTokenizer::tokenize_bool(value)
	}

	fn tokenize_bytes(value: &str) -> Result<Vec<u8>, Error> {
		StrictTokenizer::tokenize_bytes(value)
	}

	fn tokenize_fixed_bytes(value: &str, len: usize) -> Result<Vec<u8>, Error> {
		StrictTokenizer::tokenize_fixed_bytes(value, len)
	}

	fn tokenize_uint(value: &str) -> Result<[u8; 32], Error> {
		let result = StrictTokenizer::tokenize_uint(value);
		if result.is_ok() {
			return result;
		}

		let uint = Uint::from_dec_str(value)?;
		Ok(uint.into())
	}

	// We don't have a proper signed int 256-bit long type, so here we're cheating: when the
	// input is negative and fits in an i128 we return the parsed number in the 32 byte array
	// (aka `Word`) padded to the full 32 bytes. When the input is positive we build a U256 out
	// of it and check that it's within the upper bound of a hypothetical I256 type: half the
	// `U256::max_value().
	// In other words: the `int256` is parsed ok for positive values and for half the negative
	// values (`i128::min_value()`).
	fn tokenize_int(value: &str) -> Result<[u8; 32], Error> {
		let result = StrictTokenizer::tokenize_int(value);
		if result.is_ok() {
			return result;
		}
		let int = if value.starts_with("-") {
			let int = i128::from_str_radix(value, 10)?;
			pad_i128(int)
		} else {
			let pos_int = Uint::from_dec_str(value)?;
			if pos_int > Uint::max_value() / 2 {
				return Err(Error::Other("int256 parse error: Overflow".into()));
			}
			pos_int.into()
		};
		Ok(int)
	}
}
