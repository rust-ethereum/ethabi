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































/*


// Trait for functions will allow some nice meta programming.
trait EthabiFunction { // trait
    type Output;

    fn encoded(&self) -> ethabi::Bytes;

    fn output(&self, ethabi::Bytes) -> ::ethabi::Result<Self::Output>;
}

// A caller extension trait can be implemented for EthabiFunction in a separate crate.
trait DelegateCall<O> {
	fn call<C: Call<O>>(self, caller: C) -> C::Result;
	fn transact<T: Transact>(self, caller: T) -> T::Result;
}
impl<O, F: EthabiFunction<Output=O>> DelegateCall<O> for F {
    fn call<C: Call<O>>(self, caller: C) -> C::Result {
        caller.call(self.encoded(), move |bytes| self.output(bytes))
    }

    fn transact<T: Transact>(self, caller: T) -> T::Result {
        caller.transact(self.encoded())
    }
}

// Trait definitions
trait Call<Out> {
    // TODO do we actually need any bounds here?
    type Result;

    fn call<F>(self, input: ethabi::Bytes, output_decoder: F) -> Self::Result
        where F: FnOnce(ethabi::Bytes) -> Result<Out, ethabi::Error>;
}

// A blanket implementations would be nice (that's the current call signature).
// impl<Out, F> Call<Out> for F where
//     F: FnOnce(ethabi::Bytes) -> Result<ethabi::Bytes, String>
// {
//     type Result = Result<Out, ethabi::Error>;

//     fn call<F>(self, input: ethabi::Bytes, output_decoder: F) -> Self::Result {
//         (self)(input)
//             .map_err(ethabi::Error::Message)
//             .and_then(output_decoder)
//     }
// }

trait Transact {
    type Result;

    fn transact(self, ethabi::Bytes) -> Self::Result;
}

// Similar blanket impl for transact (although we don't really have bytes as result, just ())
// impl...

// Notice that futures don't need to be part of the library at all, but you can still implement asynchronous callers or transactors.

// and example of usage:

#[derive(Default)]
struct Caller;

impl<'a, Out> Call<Out> for &'a Caller {
    // I can just return results.
    type Result = Result<Out, String>;

    fn call<F>(self, input: ethabi::Bytes, output_decoder: F) -> Self::Result
        where F: FnOnce(ethabi::Bytes) -> Result<Out, ethabi::Error> {
        unimplemented!()
    }
}

use ethabi::futures::Future;

impl<'a> Transact for &'a mut Caller {
    // And here we can use futures
    type Result = Box<Future<Item = (), Error = String>>;

    fn transact(self, input: ethabi::Bytes) -> Self::Result {
        unimplemented!()
    }
}



#[test]
fn example() {
    // Caller decides on return types.
    // let caller = Caller::default();

    // // Initialize contract
    // let contract = eip20::Eip20::default();

    // // deploy contract (that additional function would be nice)
    // // (a special caller that returns address as result)
    // // let adress = contract.constructor(a, b, c).transact(caller.deploy()).wait()?;
	// let a: ethabi::Address = [2u8; 20].into();

    // // Call constant function
    // let input_bytes = contract.functions().balance_of(a)?.encoded()?;
    // let balance = contract.functions().balance_of(a)?.call(&caller)?;
    // // In case you only need to decode output use this:
    // // let balance = contract.outputs().balance_of(output_bytes)?;

    // // Transact (result dependent on the caller)
	// let b: ethabi::Address = [3u8; 20].into();
    // let receipt = contract.functions().transfer(b, 5)?.transact(&mut caller).wait()?;

    // Read events (same patter, just passing `parse_log` as decoder)
    // let transfer_logs = contract.events().transfer().create_filter(c).call(&caller)?
    //     .collect::<Vec<eip20::logs::Transfer>>();

    // // Alternatively since functions are traits, we can do this:
    // let evm = Evm::new(contract);
    // // Deploying a contract
    // evm.deploy(|contract| contract.constructor(a, b, c)).wait()?;
    // // Calling a function
    // assert_eq!(evm.call(|contract| contract.functions().balance_of(a))?, 5);
    // // Sending a transaction
    // evm.transact(|contract| contract.functions().transfer(b, 5)).wait()?;
}




*/















































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




	// #[test]
	// fn test_encoding_constructor_as_array() {
	// 	use validators::Validators;

	// 	let contract = Validators::default();
	// 	let code = Vec::new();
	// 	let first = [0x11u8; 20];
	// 	let second = [0x22u8; 20];

	// 	let encoded_from_vec = contract.constructor(code.clone(), vec![first.clone(), second.clone()]).encoded();
	// 	let encoded_from_vec_iter = contract.constructor(code.clone(), vec![first.clone(), second.clone()].into_iter()).encoded();
	// 	let encoded_from_vec_wrapped = contract.constructor(code.clone(), vec![Wrapper(first), Wrapper(second)]).encoded();

	// 	let expected = "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000011111111111111111111111111111111111111110000000000000000000000002222222222222222222222222222222222222222".to_owned();
	// 	assert_eq!(expected, encoded_from_vec.to_hex());
	// 	assert_eq!(expected, encoded_from_vec_iter.to_hex());
	// 	assert_eq!(expected, encoded_from_vec_wrapped.to_hex());
	// }

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





	/*
	OLD SYNTAX:
	contract.functions().balance_of().call(params..., callback)
	contract.functions().balance_of().input(params..., callback)
	(balance_of(). call, input, function(ethabi::function appelée en interne))

	NEW SYNTAX:
	contract.functions().balance_of(args).call(caller)

	donc pour NEW SYNTAX:
	contract.functions()

	OLD:
	impl ValidatorsFunctions {
		pub fn validators(&self) -> functions::Validators {
			functions::Validators::default()
		}
		pub fn set_validators(&self) -> functions::SetValidators {
			functions::SetValidators::default()
		}
		pub fn add_two_validators(&self) -> functions::AddTwoValidators {
			functions::AddTwoValidators::default()
		}
	}

	pub struct SetValidators {
        function: ethabi::Function,
    }

	NEW:
	impl ValidatorsFunctions {
		...
		pub fn set_validators(&self, ARG, ARG, ARG) -> functions::SetValidators {
			functions::SetValidators::default()
		}
		...
	}

	pub struct SetValidators {
        function: ethabi::Function,
		input: ...
    }

	*/


	// #[allow(dead_code)]
	// pub mod validators_file {
	// 	include!("validators_file.rs");
	// }

	#[test]
	fn test_new_syntax_constructor() {
		use validators::Validators;

		let contract = Validators::default(); // utiliser contrat généré rs

		// deploy contract (that additional function would be nice)
		// (a special caller that returns address as result)

		let code = Vec::new();
		let first = [0x11u8; 20];
		let second = [0x22u8; 20];

		let address = contract.constructor(code.clone(), vec![first.clone(), second.clone()]).transact(&|_: Bytes| Ok(/*"0000000000000000000000002222222222222222222222222222222222222222".from_hex().unwrap()*/())).unwrap();
		// assert_eq!(address, [0x22u8; 20].into());

		// use ethabi::futures::Future; // why is this needed ?
		// let address = contract.constructor(code.clone(), vec![first.clone(), second.clone()]).transact_async(&|_: Bytes| Ok(/*"0000000000000000000000002222222222222222222222222222222222222222".from_hex().unwrap()*/())).wait().unwrap();
		// assert_eq!(address, [0x22u8; 20].into());

		// let balance = contract.functions().balance_of(addr).call(&|_: Bytes| /*send bytes, return intofuture<bytes>*/ Ok("000000000000000000000000000000000000000000000000000000000000000B".from_hex().unwrap()) ).unwrap();
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
		// use ethabi::futures::Future; // why is this needed ?
		// let receipt = contract.functions().transfer(to, 5).transact_async(&|_: Bytes| /*send bytes, return intofuture<bytes>*/ Ok("0000000000000000000000000000000000000000000000000000000000000001".from_hex().unwrap())).wait().unwrap();
		// assert_eq!(receipt,()); // todo receipt
		let receipt = contract.functions().transfer(to, 5).transact(&|_: Bytes| /*send bytes, return intofuture<bytes>*/ Ok(/*"0000000000000000000000000000000000000000000000000000000000000001".from_hex().unwrap()*/())).unwrap();
		assert_eq!(receipt,());

		// Read events (same patter, just passing `parse_log` as decoder)
		/*let transfer_logs = contract.events().transfer().create_filter(c).call(&caller)?
			.collect::<Vec<eip20::logs::Transfer>>();*/

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

