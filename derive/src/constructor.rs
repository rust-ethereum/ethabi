// Copyright 2015-2019 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2::TokenStream;
use quote::quote;

use super::{
	from_template_param, get_template_names, input_names, rust_type, template_param_type, to_ethabi_param_vec, to_token,
};

/// Structure used to generate contract's constructor interface.
pub struct Constructor {
	inputs_declarations: Vec<TokenStream>,
	inputs_definitions: Vec<TokenStream>,
	tokenize: Vec<TokenStream>,
	recreate_inputs: TokenStream,
}

impl<'a> From<&'a ethabi::Constructor> for Constructor {
	fn from(c: &'a ethabi::Constructor) -> Self {
		// [param0, hello_world, param2]
		let input_names = input_names(&c.inputs);

		// [T0: Into<Uint>, T1: Into<Bytes>, T2: IntoIterator<Item = U2>, U2 = Into<Uint>]
		let inputs_declarations =
			c.inputs.iter().enumerate().map(|(index, param)| template_param_type(&param.kind, index)).collect();

		// [Uint, Bytes, Vec<Uint>]
		let kinds: Vec<_> = c.inputs.iter().map(|param| rust_type(&param.kind)).collect();

		// [T0, T1, T2]
		let template_names: Vec<_> = get_template_names(&kinds);

		// [param0: T0, hello_world: T1, param2: T2]
		let inputs_definitions = input_names
			.iter()
			.zip(template_names.iter())
			.map(|(param_name, template_name)| quote! { #param_name: #template_name });

		let inputs_definitions = Some(quote! { code: ethabi::Bytes }).into_iter().chain(inputs_definitions).collect();

		// [Token::Uint(param0.into()), Token::Bytes(hello_world.into()), Token::Array(param2.into_iter().map(Into::into).collect())]
		let tokenize: Vec<_> = input_names
			.iter()
			.zip(c.inputs.iter())
			.map(|(param_name, param)| to_token(&from_template_param(&param.kind, param_name), &param.kind))
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
	pub fn generate(&self) -> TokenStream {
		let declarations = &self.inputs_declarations;
		let definitions = &self.inputs_definitions;
		let tokenize = &self.tokenize;
		let recreate_inputs = &self.recreate_inputs;

		quote! {
			/// Encodes a call to contract's constructor.
			pub fn constructor<#(#declarations),*>(#(#definitions),*) -> ethabi::Bytes {
				let c = ethabi::Constructor {
					inputs: #recreate_inputs,
				};
				let tokens = vec![#(#tokenize),*];
				c.encode_input(code, &tokens).expect(INTERNAL_ERR)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Constructor;
	use quote::quote;

	#[test]
	fn test_no_params() {
		let ethabi_constructor = ethabi::Constructor { inputs: vec![] };

		let c = Constructor::from(&ethabi_constructor);

		let expected = quote! {
			/// Encodes a call to contract's constructor.
			pub fn constructor<>(code: ethabi::Bytes) -> ethabi::Bytes {
				let c = ethabi::Constructor {
					inputs: vec![],
				};
				let tokens = vec![];
				c.encode_input(code, &tokens).expect(INTERNAL_ERR)
			}
		};

		assert_eq!(expected.to_string(), c.generate().to_string());
	}

	#[test]
	fn test_one_param() {
		let ethabi_constructor = ethabi::Constructor {
			inputs: vec![ethabi::Param { name: "foo".into(), kind: ethabi::ParamType::Uint(256), internal_type: None }],
		};

		let c = Constructor::from(&ethabi_constructor);

		let expected = quote! {
			/// Encodes a call to contract's constructor.
			pub fn constructor<T0: Into<ethabi::Uint> >(code: ethabi::Bytes, foo: T0) -> ethabi::Bytes {
				let c = ethabi::Constructor {
					inputs: vec![ethabi::Param {
						name: "foo".to_owned(),
						kind: ethabi::ParamType::Uint(256usize),
						internal_type: None
					}],
				};
				let tokens = vec![ethabi::Token::Uint(foo.into())];
				c.encode_input(code, &tokens).expect(INTERNAL_ERR)
			}
		};

		assert_eq!(expected.to_string(), c.generate().to_string());
	}
}
