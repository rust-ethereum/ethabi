extern crate rustc_hex;
extern crate ethabi;
#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;

use rustc_hex::{ToHex, FromHex};

use_contract!(eip20, "Eip20", "../examples/eip20.json");

#[test]
fn encoding_input_works() {
	use eip20::Eip20;

	let expected = "dd62ed3e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000101010101010101010101010101010101010101".to_owned();
	let contract = Eip20::default();
	let owner = [0u8; 20];
	let spender = [1u8; 20];
	let encoded = contract.functions().allowance().input(owner, spender);
	// 4 bytes signature + 2 * 32 bytes for params
	assert_eq!(encoded.len(), 68);
	assert_eq!(encoded.to_hex(), expected);
}
