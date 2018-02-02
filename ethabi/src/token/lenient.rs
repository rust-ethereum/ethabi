use token::{Tokenizer, StrictTokenizer};
use util::pad_i32;
use errors::{Error, ErrorKind};
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

		let uint: Uint = Uint::from_dec_str(value).map_err(|_| ErrorKind::InvalidData)?;
		Ok(uint.into())
	}

	fn tokenize_int(value: &str) -> Result<[u8; 32], Error> {
		let result = StrictTokenizer::tokenize_int(value);
		if result.is_ok() {
			return result;
		}

		let int = try!(i32::from_str_radix(value, 10));
		Ok(pad_i32(int))
	}
}
