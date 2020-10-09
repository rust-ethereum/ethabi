// Copyright 2015-2019 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ethabi;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
	constructor::Constructor,
	event::Event,
	function::Function,
	options::ContractOptions
};

/// Structure used to generate rust interface for solidity contract.
pub struct Contract {
	constructor: Option<Constructor>,
	functions: Vec<Function>,
	events: Vec<Event>,
}

impl<'a> From<&'a ethabi::Contract> for Contract {
	fn from(c: &'a ethabi::Contract) -> Self {
		Contract {
			constructor: c.constructor.as_ref().map(Into::into),
			functions: c.functions().map(Into::into).collect(),
			events: c.events().map(Into::into).collect(),
		}
	}
}

impl Contract {
	pub fn new(c: &ethabi::Contract, options: Option<ContractOptions>) -> Self {

		let functions: Vec<Function> = match options {
			Some(contract_options) => {
				c.functions()
					.map(|function| {
						let mut func = Function::from(function);
						if let Some(fn_options) = contract_options.functions.get(&func.signature) {
							func.module_name = fn_options.alias.to_string();
						}
						func
					}).collect()
			},
			None => c.functions().map(Into::into).collect()
		};

		Self {
			constructor: c.constructor.as_ref().map(Into::into),
			functions,
			events: c.events().map(Into::into).collect(),
		}
	}

	/// Generates rust interface for a contract.
	pub fn generate(&self) -> TokenStream {
		let constructor = self.constructor.as_ref().map(Constructor::generate);
		let functions: Vec<_> = self.functions.iter().map(Function::generate).collect();
		let events: Vec<_> = self.events.iter().map(Event::generate_event).collect();
		let logs: Vec<_> = self.events.iter().map(Event::generate_log).collect();
		quote! {
			use ethabi;
			const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";

			#constructor

			/// Contract's functions.
			pub mod functions {
				use super::INTERNAL_ERR;
				#(#functions)*
			}

			/// Contract's events.
			pub mod events {
				use super::INTERNAL_ERR;
				#(#events)*
			}

			/// Contract's logs.
			pub mod logs {
				use super::INTERNAL_ERR;
				use ethabi;
				#(#logs)*
			}
		}
	}
}

#[cfg(test)]
mod test {
	use ethabi;
	use quote::quote;

	use super::Contract;

	#[test]
	fn test_no_body() {
		let ethabi_contract = ethabi::Contract {
			constructor: None,
			functions: Default::default(),
			events: Default::default(),
			fallback: false,
		};

		let c = Contract::from(&ethabi_contract);

		let expected = quote! {
			use ethabi;
			const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";

			/// Contract's functions.
			pub mod functions {
				use super::INTERNAL_ERR;
			}

			/// Contract's events.
			pub mod events {
				use super::INTERNAL_ERR;
			}

			/// Contract's logs.
			pub mod logs {
				use super::INTERNAL_ERR;
				use ethabi;
			}
		};

		assert_eq!(expected.to_string(), c.generate().to_string());
	}
}
