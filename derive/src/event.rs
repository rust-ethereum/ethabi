use {quote, syn, ethabi};
use heck::{SnakeCase, CamelCase};

use super::{rust_type};

pub struct Event {
	name: String,
	log_fields: Vec<quote::Tokens>,
}

impl<'a> From<&'a ethabi::Event> for Event {
	fn from(e: &'a ethabi::Event) -> Self {
		let names: Vec<_> = e.inputs
			.iter()
			.enumerate()
			.map(|(index, param)| if param.name.is_empty() {
				syn::Ident::from(format!("param{}", index))
			} else {
				param.name.to_snake_case().into()
			}).collect();
		let kinds: Vec<_> = e.inputs
			.iter()
			.map(|param| rust_type(&param.kind))
			.collect();
		let log_fields= names.iter().zip(kinds.iter())
			.map(|(param_name, kind)| quote! { pub #param_name: #kind })
			.collect();

		Event {
			name: e.name.clone(),
			log_fields,
		}
	}
}

impl Event {
	pub fn generate_log(&self) -> quote::Tokens {
		let name = syn::Ident::from(self.name.to_camel_case());
		let log_fields = &self.log_fields;

		quote! {
			#[derive(Debug, Clone, PartialEq)]
			pub struct #name {
				#(#log_fields),*
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use ethabi;
	use super::Event;

	#[test]
	fn test_empty_log() {
		let ethabi_event = ethabi::Event {
			name: "hello".into(),
			inputs: vec![],
			anonymous: false,
		};

		let e = Event::from(&ethabi_event);

		let expected = quote! {
			#[derive(Debug, Clone, PartialEq)]
			pub struct Hello {}
		};

		assert_eq!(expected, e.generate_log());
	}

	#[test]
	fn test_log_with_one_field() {
		let ethabi_event = ethabi::Event {
			name: "one".into(),
			inputs: vec![ethabi::EventParam {
				name: "foo".into(),
				kind: ethabi::ParamType::Address,
				indexed: false
			}],
			anonymous: false,
		};

		let e = Event::from(&ethabi_event);

		let expected = quote! {
			#[derive(Debug, Clone, PartialEq)]
			pub struct One {
				pub foo: ethabi::Address
			}
		};

		assert_eq!(expected, e.generate_log());
	}

	#[test]
	fn test_log_with_multiple_field() {
		let ethabi_event = ethabi::Event {
			name: "many".into(),
			inputs: vec![ethabi::EventParam {
				name: "foo".into(),
				kind: ethabi::ParamType::Address,
				indexed: false
			}, ethabi::EventParam {
				name: "bar".into(),
				kind: ethabi::ParamType::Array(Box::new(ethabi::ParamType::String)),
				indexed: false
			}, ethabi::EventParam {
				name: "xyz".into(),
				kind: ethabi::ParamType::Uint(256),
				indexed: false
			}],
			anonymous: false,
		};

		let e = Event::from(&ethabi_event);

		let expected = quote! {
			#[derive(Debug, Clone, PartialEq)]
			pub struct Many {
				pub foo: ethabi::Address,
				pub bar: Vec<String>,
				pub xyz: ethabi::Uint
			}
		};

		assert_eq!(expected, e.generate_log());
	}
}
