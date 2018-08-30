use {quote, syn, ethabi};
use heck::{SnakeCase, CamelCase};

use super::{rust_type, to_syntax_string, from_token, get_template_names, to_token};

pub struct Event {
	name: String,
	log_fields: Vec<quote::Tokens>,
	recreate_inputs_quote: quote::Tokens,
	log_init: Vec<quote::Tokens>,
	wildcard_filter_params: Vec<quote::Tokens>,
	filter_declarations: Vec<quote::Tokens>,
	filter_definitions: Vec<quote::Tokens>,
	filter_init: Vec<quote::Tokens>,
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

		let topic_kinds: Vec<_> = e.inputs
			.iter()
			.filter(|param| param.indexed)
			.map(|param| rust_type(&param.kind))
			.collect();
		let topic_names: Vec<_> = e.inputs
			.iter()
			.enumerate()
			.filter(|&(_, param)| param.indexed)
			.map(|(index, param)| if param.name.is_empty() {
				syn::Ident::from(format!("topic{}", index))
			} else {
				param.name.to_snake_case().into()
			})
			.collect();

		// [T0, T1, T2]
		let template_names: Vec<_> = get_template_names(&topic_kinds);

		let filter_declarations: Vec<_> = topic_kinds.iter().zip(template_names.iter())
			.map(|(kind, template_name)| quote! { #template_name: Into<ethabi::Topic<#kind>> })
			.collect();

		let filter_definitions: Vec<_> = topic_names.iter().zip(template_names.iter())
			.map(|(param_name, template_name)| quote! { #param_name: #template_name })
			.collect();

		// The number of parameters that creates a filter which matches anything.
		let wildcard_filter_params: Vec<_> = filter_definitions.iter().map(|_| quote! { ethabi::Topic::Any })
			.collect();

		let filter_init: Vec<_> = topic_names.iter().zip(e.inputs.iter().filter(|p| p.indexed))
			.enumerate()
			.take(3)
			.map(|(index, (param_name, param))| {
				let topic = syn::Ident::from(format!("topic{}", index));
				let i = quote! { i };
				let to_token = to_token(&i, &param.kind);
				quote! { #topic: #param_name.into().map(|#i| #to_token), }
			})
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
			wildcard_filter_params,
			filter_declarations,
			filter_definitions,
			filter_init,
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
		let filter_init = &self.filter_init;
		let filter_declarations = &self.filter_declarations;
		let filter_definitions = &self.filter_definitions;
		let wildcard_filter_params = &self.wildcard_filter_params;

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


				pub fn filter<#(#filter_declarations),*>(#(#filter_definitions),*) -> ethabi::TopicFilter {
					let raw = ethabi::RawTopicFilter {
						#(#filter_init)*
						..Default::default()
					};

					self.event.filter(raw).expect(INTERNAL_ERR)
				}

				pub fn wildcard_filter() -> ethabi::TopicFilter {
					self.filter(#(#wildcard_filter_params),*)
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

				pub fn filter<>() -> ethabi::TopicFilter {
					let raw = ethabi::RawTopicFilter {
						..Default::default()
					};

					self.event.filter(raw).expect(INTERNAL_ERR)
				}

				pub fn wildcard_filter() -> ethabi::TopicFilter {
					self.filter()
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
	fn test_event_with_one_input() {
		let ethabi_event = ethabi::Event {
			name: "one".into(),
			inputs: vec![ethabi::EventParam {
				name: "foo".into(),
				kind: ethabi::ParamType::Address,
				indexed: true
			}],
			anonymous: false,
		};

		let e = Event::from(&ethabi_event);

		let expected = quote! {
			pub mod one {
				use ethabi;
				use super::INTERNAL_ERR;

				struct Event {
					event: ethabi::Event,
				}

				impl Default for Event {
					fn default() -> Self {
						Event {
							event: "One".into(),
							inputs: vec![ethabi::EventParam {
								name: "foo".to_owned(),
								kind: ethabi::ParamType::Address,
								indexed: true
							}],
							anonymous: false,
						}
					}
				}

				pub fn filter<T0: Into<ethabi::Topic<ethabi::Address>>>(foo: T0) -> ethabi::TopicFilter {
					let raw = ethabi::RawTopicFilter {
						topic0: foo.into().map(|i| ethabi::Token::Address(i)),
						..Default::default()
					};

					self.event.filter(raw).expect(INTERNAL_ERR)
				}

				pub fn wildcard_filter() -> ethabi::TopicFilter {
					self.filter(ethabi::Topic::Any)
				}

				pub fn parse_log(log: ethabi::RawLog) -> ethabi::Result<super::logs::One> {
					let e = Event::default();
					let mut log = e.parse_log(log)?.params.into_iter();
					let result = super::logs::One {
						foo: log.next().expect(INTERNAL_ERR).value.to_address().expect(INTERNAL_ERR)
					};
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
