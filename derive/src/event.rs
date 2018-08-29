use {quote, syn, ethabi};
use heck::{SnakeCase, CamelCase};

use super::{rust_type, to_syntax_string, from_token};

pub struct Event {
	name: String,
	log_fields: Vec<quote::Tokens>,
	recreate_inputs_quote: quote::Tokens,
	log_init: Vec<quote::Tokens>,
	anonymous: bool,
}

impl<'a> From<&'a ethabi::Event> for Event {
	fn from(e: &'a ethabi::Event) -> Self {
		let names: Vec<_> = e.inputs
			.iter()
			.enumerate()
			.map(|(index, param)| if param.name.is_empty() {
				if param.indexed {
					syn::Ident::from(format!("topic{}", index))
				} else {
					syn::Ident::from(format!("param{}", index))
				}
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

		let log_iter = quote! { log.next().expect(INTERNAL_ERR).value };

		let to_log: Vec<_> = e.inputs
			.iter()
			.map(|param| from_token(&param.kind, &log_iter))
			.collect();

		let log_init = names.iter().zip(to_log.iter())
			.map(|(param_name, convert)| quote! { #param_name: #convert })
			.collect();

		let event_inputs = &e.inputs.iter().map(|x| {
			let name = &x.name;
			let kind = to_syntax_string(&x.kind);
			let indexed = x.indexed;

			quote! {
				ethabi::EventParam {
					name: #name.to_owned(),
					kind: #kind,
					indexed: #indexed
				}
			}
		}).collect::<Vec<_>>();
		let recreate_inputs_quote = quote! { vec![ #(#event_inputs),* ] };

		Event {
			name: e.name.clone(),
			log_fields,
			recreate_inputs_quote,
			log_init,
			anonymous: e.anonymous,
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

	pub fn generate_event(&self) -> quote::Tokens {
		let name_as_string = &self.name.to_camel_case();
		let name = syn::Ident::from(self.name.to_snake_case());
		let camel_name = syn::Ident::from(self.name.to_camel_case());
		let recreate_inputs_quote = &self.recreate_inputs_quote;
		let anonymous = &self.anonymous;
		let log_init = &self.log_init;

		quote! {
			pub mod #name {
				use ethabi;
				use super::INTERNAL_ERR;

				struct Event {
					event: ethabi::Event,
				}

				impl Default for Event {
					fn default() -> Self {
						Event {
							event: #name_as_string.into(),
							inputs: #recreate_inputs_quote,
							anonymous: #anonymous,
						}
					}
				}

				pub fn filter() -> ethabi::TopicFilter {
				}

				pub fn wildcard_filter() -> ethabi::TopicFilter {
				}

				pub fn parse_log(log: ethabi::RawLog) -> ethabi::Result<super::logs::#camel_name> {
					let e = Event::default();
					let mut log = e.parse_log(log)?.params.into_iter();
					let result = super::logs::#camel_name {
						#(#log_init),*
					};
					Ok(result)
				}
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
	fn test_empty_event() {
		let ethabi_event = ethabi::Event {
			name: "hello".into(),
			inputs: vec![],
			anonymous: false,
		};

		let e = Event::from(&ethabi_event);

		let expected = quote! {
			pub mod hello {
				use ethabi;
				use super::INTERNAL_ERR;

				struct Event {
					event: ethabi::Event,
				}

				impl Default for Event {
					fn default() -> Self {
						Event {
							event: "Hello".into(),
							inputs: vec![],
							anonymous: false,
						}
					}
				}

				pub fn filter() -> ethabi::TopicFilter {
				}

				pub fn wildcard_filter() -> ethabi::TopicFilter {
				}

				pub fn parse_log(log: ethabi::RawLog) -> ethabi::Result<super::logs::Hello> {
					let e = Event::default();
					let mut log = e.parse_log(log)?.params.into_iter();
					let result = super::logs::Hello {};
					Ok(result)
				}
			}
		};

		assert_eq!(expected, e.generate_event());
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
