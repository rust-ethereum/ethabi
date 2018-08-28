use {syn, quote, ethabi};
use function::Function;

pub struct Contract {
	name: String,
	functions: Vec<Function>,
}

impl Contract {
	fn new(name: String, c: &ethabi::Contract) -> Contract {
		Contract {
			name,
			functions: c.functions().map(Into::into).collect(),
		}
	}

	fn generate(&self) -> quote::Tokens {
		let module_name = syn::Ident::from(&self.name as &str);
		quote! {
			pub mod #module_name {
			}
		}
	}
}

#[cfg(test)]
mod test {
	use ethabi;
	use super::Contract;

	#[test]
	fn test() {
		let ethabi_contract = ethabi::Contract {
			constructor: None,
			functions: Default::default(),
			events: Default::default(),
			fallback: false,
		};

		let c = Contract::new("foo".into(), &ethabi_contract);

		let expected = quote! {
			pub mod foo {
			}
		};

		assert_eq!(expected, c.generate());
	}
}
