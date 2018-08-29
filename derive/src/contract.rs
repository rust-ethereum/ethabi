use {syn, quote, ethabi};
use function::Function;
use event::Event;

pub struct Contract {
	name: String,
	functions: Vec<Function>,
	events: Vec<Event>,
}

impl Contract {
	fn new(name: String, c: &ethabi::Contract) -> Contract {
		Contract {
			name,
			functions: c.functions().map(Into::into).collect(),
			events: c.events().map(Into::into).collect(),
		}
	}

	fn generate(&self) -> quote::Tokens {
		let module_name = syn::Ident::from(&self.name as &str);
		let functions: Vec<_> = self.functions.iter().map(Function::generate).collect();
		let events: Vec<_> = self.events.iter().map(Event::generate_event).collect();
		let logs: Vec<_> = self.events.iter().map(Event::generate_log).collect();
		quote! {
			pub mod #module_name {
				pub mod functions {
					#(#functions)*
				}

				pub mod events {
					#(#events)*
				}

				pub mod logs {
					#(#logs)*
				}
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

		let c = Contract::new("foo".into(), &ethabi_contract);

		let expected = quote! {
			pub mod foo {
				pub mod functions {
				}

				pub mod events {
				}

				pub mod logs {
				}
			}
		};

		assert_eq!(expected, c.generate());
	}
}
