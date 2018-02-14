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

use_contract!(eip20, "Eip20", "../res/eip20.abi");
use_contract!(constructor, "Constructor", "../res/con.abi");
use_contract!(validators, "Validators", "../res/Validators.abi");
use_contract!(operations, "Operations", "../res/Operations.abi");

#[cfg(test)]
mod tests {
	use rustc_hex::{ToHex, FromHex};
	use ethabi::{Address, Uint, Bytes, EthabiFunction, DelegateCall};

	struct Wrapper([u8; 20]);

	impl Into<Address> for Wrapper {
		fn into(self) -> Address {
			self.0.into()
		}
	}

	#[test]
	fn test_encoding_function_input_as_array() {
		use validators::Validators;

		let contract = Validators::default();
		let first = [0x11u8; 20];
		let second = [0x22u8; 20];

		let functions = contract.functions();

		let encoded_from_vec = functions.set_validators(vec![first.clone(), second.clone()]).encoded();
		let encoded_from_vec_iter = functions.set_validators(vec![first.clone(), second.clone()].into_iter()).encoded();
		let encoded_from_vec_wrapped = functions.set_validators(vec![Wrapper(first), Wrapper(second)]).encoded();

		let expected = "9300c9260000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000011111111111111111111111111111111111111110000000000000000000000002222222222222222222222222222222222222222".to_owned();
		assert_eq!(expected, encoded_from_vec.to_hex());
		assert_eq!(expected, encoded_from_vec_iter.to_hex());
		assert_eq!(expected, encoded_from_vec_wrapped.to_hex());
	}

	#[test]
	fn test_decoding_function_output() {
		// Make sure that the output param type of the derived contract is correct

		use eip20::Eip20;

		let contract = Eip20::default();
		let output = "000000000000000000000000000000000000000000000000000000000036455B".from_hex().unwrap();
		let decoded_output = contract.outputs().total_supply(&output).unwrap();
		let expected_output: Uint = 0x36455b.into();
		assert_eq!(expected_output, decoded_output);
	}

	#[test]
	fn test_constructor_transaction() {
		use validators::Validators;

		let contract = Validators::default();

		// deploy contract (that additional function would be nice)
		// (a special caller that returns address as result)

		let code = Vec::new();
		let first = [0x11u8; 20];
		let second = [0x22u8; 20];

		let address = contract.constructor(code.clone(), vec![first.clone(), second.clone()]).transact(&|_: Bytes| Ok(())).unwrap();
		assert_eq!(address, ());

		// Todo make transact async work
		// use ethabi::futures::Future;
		// let address = contract.constructor(code.clone(), vec![first.clone(), second.clone()]).transact(&|_: Bytes| Future::ok::<u32, u32>(1)).wait().unwrap();
		// assert_eq!(address, ());
	}

	#[test]
	fn test_encoding_constructor_as_array() {
		use validators::Validators;

		let contract = Validators::default();
		let code = Vec::new();
		let first = [0x11u8; 20];
		let second = [0x22u8; 20];

		let encoded_from_vec = contract.constructor(code.clone(), vec![first.clone(), second.clone()]).encoded();
		let encoded_from_vec_iter = contract.constructor(code.clone(), vec![first.clone(), second.clone()].into_iter()).encoded();
		let encoded_from_vec_wrapped = contract.constructor(code.clone(), vec![Wrapper(first), Wrapper(second)]).encoded();

		let expected = "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000011111111111111111111111111111111111111110000000000000000000000002222222222222222222222222222222222222222".to_owned();
		assert_eq!(expected, encoded_from_vec.to_hex());
		assert_eq!(expected, encoded_from_vec_iter.to_hex());
		assert_eq!(expected, encoded_from_vec_wrapped.to_hex());
	}

	#[test]
	fn test_encoding_function_input_as_fixed_array() {
		use validators::Validators;

		let contract = Validators::default();
		let first = [0x11u8; 20];
		let second = [0x22u8; 20];

		let functions = contract.functions();

		let encoded_from_array = functions.add_two_validators([first.clone(), second.clone()]).encoded();
		let encoded_from_array_wrapped = functions.add_two_validators([Wrapper(first), Wrapper(second)]).encoded();

		let expected = "7de33d2000000000000000000000000011111111111111111111111111111111111111110000000000000000000000002222222222222222222222222222222222222222".to_owned();
		assert_eq!(expected, encoded_from_array.to_hex());
		assert_eq!(expected, encoded_from_array_wrapped.to_hex());
	}

	#[test]
	fn encoding_input_works() {
		use eip20::Eip20;

		let expected = "dd62ed3e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000101010101010101010101010101010101010101".to_owned();
		let contract = Eip20::default();
		let owner = [0u8; 20];
		let spender = [1u8; 20];
		let encoded = contract.functions().allowance(owner, spender).encoded();
		// 4 bytes signature + 2 * 32 bytes for params
		assert_eq!(encoded.to_hex(), expected);

		let from: Address = [2u8; 20].into();
		let to: Address = [3u8; 20].into();
		let to2: Address = [4u8; 20].into();
		let _filter = contract.events().transfer().create_filter(from, vec![to, to2]);
		let _filter = contract.events().transfer().create_filter(None, None);
	}

	#[test]
	fn test_calling_function() {
		use eip20::Eip20;

		let contract = Eip20::default();
		let address_param = [0u8; 20];
		let result = contract.functions().balance_of(address_param).call(&|data| {
			assert_eq!(data, "70a082310000000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap());
			Ok("000000000000000000000000000000000000000000000000000000000036455b".from_hex().unwrap())
        });
		assert_eq!(result.unwrap(), "000000000000000000000000000000000000000000000000000000000036455b".into());
	}

	#[allow(dead_code)]
	pub mod eip20_file {
		include!("eip20_file.rs");
	}

	#[test]
	fn test_new_syntax() {
		// Caller decides on return types.
		// let caller = Caller::default();

		// Initialize contract

		let contract = eip20_file::Eip20::default(); // utiliser contrat généré rs

		// deploy contract (that additional function would be nice)
		// (a special caller that returns address as result)
		// TODO : let address = contract.constructor(a, b, c).transact(caller.deploy()).wait()?;

		// Call constant function
		let addr: Address = [2u8; 20].into();
		let input_bytes = contract.functions().balance_of(addr).encoded();
		println!("input_bytes: {:?}", input_bytes);

// R: futures::IntoFuture<Item=Bytes, Error=String> + Send,
	//F: FnOnce(Bytes) -> R,
		let balance = contract.functions().balance_of(addr).call(&|_: Bytes| /*send bytes, return intofuture<bytes>*/ Ok("000000000000000000000000000000000000000000000000000000000000000B".from_hex().unwrap()) ).unwrap();
		println!("balance: {:?}", balance);
		assert_eq!(balance,11.into());

		// In case you only need to decode output use this:
		let output_bytes = "000000000000000000000000000000000000000000000000000000000036455B".from_hex().unwrap();
		let balance = contract.outputs().balance_of(&output_bytes).unwrap();
		println!("balance: {:?}", balance);

		// Transact (result dependent on the caller)
		let to: Address = [3u8; 20].into();
		use ethabi::futures::Future; // why is this needed ?
		let receipt = contract.functions().transfer(to, 5).transact(&|_: Bytes| /*ou wait future ok*/ Ok("0000000000000000000000000000000000000000000000000000000000000001".from_hex().unwrap())/* as future::IntoFuture*/).wait().unwrap();
		assert_eq!(receipt,());
		let receipt = contract.functions().transfer(to, 5).transact(&|_: Bytes| Ok(/*"0000000000000000000000000000000000000000000000000000000000000001".from_hex().unwrap()*/())).unwrap();
		assert_eq!(receipt,());

		// Read events (same patter, just passing `parse_log` as decoder)
		// TODO does not work
		// let _transfer_logs = contract.events().transfer().create_filter(c).call(&|_: Bytes| Ok("000000000000000000000000000000000000000000000000000000000000000B".from_hex().unwrap())).unwrap()
		// 	.collect::<Vec<eip20_file::Eip20::logs::Transfer>>();

		/*
		// Alternatively since functions are traits, we can do this:
		let evm = Evm::new(contract);
		// Deploying a contract
		evm.deploy(|contract| contract.constructor(a, b, c)).wait()?;
		// Calling a function
		assert_eq!(evm.call(|contract| contract.functions().balance_of(a))?, 5);
		// Sending a transaction
		evm.transact(|contract| contract.functions().transfer(b, 5)).wait()?;
		*/
	}

}

