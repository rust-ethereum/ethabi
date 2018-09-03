use {quote, ethabi};

use super::{
	input_names, template_param_type, rust_type, get_template_names, to_token, from_template_param,
	to_ethabi_param_vec,
};

/// Structure used to generate contract's construtor interface.
pub struct Constructor {
	inputs_declarations: Vec<quote::Tokens>,
	inputs_definitions: Vec<quote::Tokens>,
	tokenize: Vec<quote::Tokens>,
	recreate_inputs: quote::Tokens,
}

impl<'a> From<&'a ethabi::Constructor> for Constructor {
	fn from(c: &'a ethabi::Constructor) -> Self {
		// [param0, hello_world, param2]
		let input_names = input_names(&c.inputs);

		// [T0: Into<Uint>, T1: Into<Bytes>, T2: IntoIterator<Item = U2>, U2 = Into<Uint>]
		let inputs_declarations = c.inputs.iter().enumerate()
			.map(|(index, param)| template_param_type(&param.kind, index))
			.collect();

		// [Uint, Bytes, Vec<Uint>]
		let kinds: Vec<_> = c.inputs
			.iter()
			.map(|param| rust_type(&param.kind))
			.collect();

		// [T0, T1, T2]
		let template_names: Vec<_> = get_template_names(&kinds);

		// [param0: T0, hello_world: T1, param2: T2]
		let inputs_definitions = input_names.iter().zip(template_names.iter())
			.map(|(param_name, template_name)| quote! { #param_name: #template_name });

		let inputs_definitions = Some(quote! { code: ethabi::Bytes }).into_iter()
			.chain(inputs_definitions)
			.collect();

		// [Token::Uint(param0.into()), Token::Bytes(hello_world.into()), Token::Array(param2.into_iter().map(Into::into).collect())]
		let tokenize: Vec<_> = input_names.iter().zip(c.inputs.iter())
			.map(|(param_name, param)| to_token(&from_template_param(&param.kind, &param_name), &param.kind))
			.collect();

		Constructor {
			inputs_declarations,
			inputs_definitions,
			tokenize,
			recreate_inputs: to_ethabi_param_vec(&c.inputs),
		}
	}
}

impl Constructor {
	/// Generates contract constructor interface.
	pub fn generate(&self) -> quote::Tokens {
		let declarations = &self.inputs_declarations;
		let definitions = &self.inputs_definitions;
		let tokenize = &self.tokenize;
		let recreate_inputs = &self.recreate_inputs;

		quote! {
			struct Constructor {
				constructor: ethabi::Constructor,
			}

			impl Default for Constructor {
				fn default() -> Self {
					Constructor {
						constructor: ethabi::Constructor {
							inputs: #recreate_inputs,
						}
					}
				}
			}

			pub fn constructor<#(#declarations),*>(#(#definitions),*) -> ethabi::Bytes {
				let c = Constructor::default();
				let tokens = vec![#(#tokenize),*];
				c.constructor.encode_input(code, &tokens).expect(INTERNAL_ERR)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use ethabi;
	use super::Constructor;

	#[test]
	fn test_no_params() {
		let ethabi_constructor = ethabi::Constructor {
			inputs: vec![],
		};

		let c = Constructor::from(&ethabi_constructor);

		let expected = quote! {
			struct Constructor {
				constructor: ethabi::Constructor,
			}

			impl Default for Constructor {
				fn default() -> Self {
					Constructor {
						constructor: ethabi::Constructor {
							inputs: vec![],
						}
					}
				}
			}

			pub fn constructor<>(code: ethabi::Bytes) -> ethabi::Bytes {
				let c = Constructor::default();
				let tokens = vec![];
				c.constructor.encode_input(code, &tokens).expect(INTERNAL_ERR)
			}
		};

		assert_eq!(expected, c.generate());
	}

	#[test]
	fn test_one_param() {
		let ethabi_constructor = ethabi::Constructor {
			inputs: vec![
				ethabi::Param {
					name: "foo".into(),
					kind: ethabi::ParamType::Uint(256),
				}
			],
		};

		let c = Constructor::from(&ethabi_constructor);

		let expected = quote! {
			struct Constructor {
				constructor: ethabi::Constructor,
			}

			impl Default for Constructor {
				fn default() -> Self {
					Constructor {
						constructor: ethabi::Constructor {
							inputs: vec![ethabi::Param {
								name: "foo".to_owned(),
								kind: ethabi::ParamType::Uint(256usize)
							}],
						}
					}
				}
			}

			pub fn constructor<T0: Into<ethabi::Uint> >(code: ethabi::Bytes, foo: T0) -> ethabi::Bytes {
				let c = Constructor::default();
				let tokens = vec![ethabi::Token::Uint(foo.into())];
				c.constructor.encode_input(code, &tokens).expect(INTERNAL_ERR)
			}
		};

		assert_eq!(expected, c.generate());
	}
}
