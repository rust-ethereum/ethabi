use {quote, ethabi};
use function::Function;
use event::Event;

pub struct Contract {
	functions: Vec<Function>,
	events: Vec<Event>,
}

impl<'a> From<&'a ethabi::Contract> for Contract {
	fn from(c: &'a ethabi::Contract) -> Self {
		Contract {
			functions: c.functions().map(Into::into).collect(),
			events: c.events().map(Into::into).collect(),
		}
	}
}

impl Contract {
	pub fn generate(&self) -> quote::Tokens {
		let functions: Vec<_> = self.functions.iter().map(Function::generate).collect();
		let events: Vec<_> = self.events.iter().map(Event::generate_event).collect();
		let logs: Vec<_> = self.events.iter().map(Event::generate_log).collect();
		quote! {
			const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";

			pub mod functions {
				use super::INTERNAL_ERR;
				#(#functions)*
			}

			pub mod events {
				use super::INTERNAL_ERR;
				#(#events)*
			}

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
			const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";
			pub mod functions {
				use super::INTERNAL_ERR;
			}

			pub mod events {
				use super::INTERNAL_ERR;
			}

			pub mod logs {
				use super::INTERNAL_ERR;
				use ethabi;
			}
		};

		assert_eq!(expected, c.generate());
	}
}
