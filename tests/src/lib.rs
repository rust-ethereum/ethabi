//! Test crate

#![deny(missing_docs)]
#![deny(dead_code)]
#![deny(unused_imports)]

extern crate rustc_hex;
extern crate ethabi;
#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;

use_contract!(eip20, "../res/eip20.abi");
use_contract!(constructor, "../res/constructor.abi");
use_contract!(validators, "../res/Validators.abi");
use_contract!(operations, "../res/Operations.abi");
use_contract!(urlhint, "../res/urlhint.abi");
use_contract!(test_rust_keywords, "../res/test_rust_keywords.abi");

#[cfg(test)]
mod tests {
	use rustc_hex::{ToHex, FromHex};
	use ethabi::{Address, Uint, ContractFunction};

	struct Wrapper([u8; 20]);

	impl Into<Address> for Wrapper {
		fn into(self) -> Address {
			self.0.into()
		}
	}

	#[test]
	fn test_encoding_function_input_as_array() {
        use validators::functions;

		let first = [0x11u8; 20];
		let second = [0x22u8; 20];

		let encoded_from_vec = functions::set_validators(vec![first.clone(), second.clone()]).encoded();
		let encoded_from_vec_iter = functions::set_validators(vec![first.clone(), second.clone()].into_iter()).encoded();
		let encoded_from_vec_wrapped = functions::set_validators(vec![Wrapper(first), Wrapper(second)]).encoded();

		let expected = "9300c9260000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000011111111111111111111111111111111111111110000000000000000000000002222222222222222222222222222222222222222".to_owned();
		assert_eq!(expected, encoded_from_vec.to_hex());
		assert_eq!(expected, encoded_from_vec_iter.to_hex());
		assert_eq!(expected, encoded_from_vec_wrapped.to_hex());
	}

	#[test]
	fn test_decoding_function_output() {
		// Make sure that the output param type of the derived contract is correct

		// given

		use eip20;
		let output = "000000000000000000000000000000000000000000000000000000000036455B".from_hex().unwrap();

		// when

		let decoded_output = eip20::outputs::total_supply(&output).unwrap();
		let decoded_output2 = eip20::functions::total_supply().output(output).unwrap();

		// then

		let expected_output: Uint = 0x36455b.into();
		assert_eq!(expected_output, decoded_output);
		assert_eq!(expected_output, decoded_output2);
	}

	#[test]
	fn test_encoding_constructor_as_array() {
		use validators::constructor;

		let code = Vec::new();
		let first = [0x11u8; 20];
		let second = [0x22u8; 20];

		let encoded_from_vec = constructor(code.clone(), vec![first.clone(), second.clone()]).encoded();
		let encoded_from_vec_iter = constructor(code.clone(), vec![first.clone(), second.clone()].into_iter()).encoded();
		let encoded_from_vec_wrapped = constructor(code.clone(), vec![Wrapper(first), Wrapper(second)]).encoded();

		let expected = "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000011111111111111111111111111111111111111110000000000000000000000002222222222222222222222222222222222222222".to_owned();
		assert_eq!(expected, encoded_from_vec.to_hex());
		assert_eq!(expected, encoded_from_vec_iter.to_hex());
		assert_eq!(expected, encoded_from_vec_wrapped.to_hex());
	}

	#[test]
	fn test_encoding_function_input_as_fixed_array() {
		use validators::functions;

		let first = [0x11u8; 20];
		let second = [0x22u8; 20];

		let encoded_from_array = functions::add_two_validators([first.clone(), second.clone()]).encoded();
		let encoded_from_array_wrapped = functions::add_two_validators([Wrapper(first), Wrapper(second)]).encoded();
		let encoded_from_string = functions::set_title("foo").encoded();

		let expected_array = "7de33d2000000000000000000000000011111111111111111111111111111111111111110000000000000000000000002222222222222222222222222222222222222222".to_owned();
		let expected_string = "72910be000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003666f6f0000000000000000000000000000000000000000000000000000000000".to_owned();
		assert_eq!(expected_array, encoded_from_array.to_hex());
		assert_eq!(expected_array, encoded_from_array_wrapped.to_hex());
		assert_eq!(expected_string, encoded_from_string.to_hex())
	}

	#[test]
	fn encoding_input_works() {
		use eip20;
		use ethabi::LogFilter;

		let expected = "dd62ed3e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000101010101010101010101010101010101010101".to_owned();
		let owner = [0u8; 20];
		let spender = [1u8; 20];
		let encoded = eip20::functions::allowance(owner, spender).encoded();
		// 4 bytes signature + 2 * 32 bytes for params
		assert_eq!(encoded.to_hex(), expected);

		let from: Address = [2u8; 20].into();
		let to: Address = [3u8; 20].into();
		let to2: Address = [4u8; 20].into();
		let _filter = eip20::events::transfer().filter(from, vec![to, to2]);
		let wildcard_filter = eip20::events::transfer().filter(None, None);
		let wildcard_filter_sugared = eip20::events::transfer().wildcard_filter();
		assert_eq!(wildcard_filter, wildcard_filter_sugared);
	}
}

