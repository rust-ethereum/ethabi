#![recursion_limit="256"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate heck;
extern crate ethabi;

use std::{env, fs};
use std::path::PathBuf;
use proc_macro::TokenStream;
use heck::{SnakeCase, CamelCase};
use ethabi::{Result, ResultExt, Contract, Event, Function, ParamType, Constructor};

const ERROR_MSG: &'static str = "`derive(EthabiContract)` failed";

#[proc_macro_derive(EthabiContract, attributes(ethabi_contract_options))]
pub fn ethabi_derive(input: TokenStream) -> TokenStream {
	let s = input.to_string();
	let ast = syn::parse_derive_input(&s).expect(ERROR_MSG);
	let gen = impl_ethabi_derive(&ast).expect(ERROR_MSG);
	gen.parse().expect(ERROR_MSG)
}

fn impl_ethabi_derive(ast: &syn::DeriveInput) -> Result<quote::Tokens> {
	let options = get_options(&ast.attrs, "ethabi_contract_options")?;
	let path = get_option(&options, "path")?;
	let normalized_path = normalize_path(path)?;
	let source_file = fs::File::open(&normalized_path)
		.chain_err(|| format!("Cannot load contract abi from `{}`", normalized_path.display()))?;
	let contract = Contract::load(source_file)?;

	let functions: Vec<_> = contract.functions().map(impl_contract_function).collect();
	let events_impl: Vec<_> = contract.events().map(impl_contract_event).collect();
	let constructor_impl = contract.constructor.as_ref().map(impl_contract_constructor);
	let logs_structs: Vec<_> = contract.events().map(declare_logs).collect();
	let events_structs: Vec<_> = contract.events().map(declare_events).collect();
	let func_structs: Vec<_> = contract.functions().map(declare_functions).collect();

	let name = get_option(&options, "name")?;
	let name = syn::Ident::new(name);
	let functions_name = syn::Ident::new(format!("{}Functions", name));
	let events_name = syn::Ident::new(format!("{}Events", name));

	let file_path = string_ident(&normalized_path.display().to_string());

	let result = quote! {
		use ethabi;

		const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";

		pub mod logs {
			use ethabi;
			#(#logs_structs)*
		}

		pub mod events {
			use ethabi;

			#(#events_structs)*
		}

		pub mod functions {
			use ethabi;

			#(#func_structs)*
		}

		pub struct #functions_name<'a> {
			contract: &'a ethabi::Contract,
		}

		impl<'a> #functions_name<'a> {
			#(#functions)*
		}

		pub struct #events_name<'a> {
			contract: &'a ethabi::Contract,
		}

		impl<'a> #events_name<'a> {
			#(#events_impl)*
		}

		/// Contract
		pub struct #name {
			contract: ethabi::Contract,
		}

		impl Default for #name {
			fn default() -> Self {
				let contract = ethabi::Contract::load(include_bytes!(#file_path) as &[u8]).expect(INTERNAL_ERR);
				#name {
					contract,
				}
			}
		}

		impl #name {
			pub fn functions(&self) -> #functions_name {
				#functions_name {
					contract: &self.contract,
				}
			}

			pub fn events(&self) -> #events_name {
				#events_name {
					contract: &self.contract,
				}
			}

			#constructor_impl
		}

	};
	Ok(result)
}

fn string_ident(s: &str) -> syn::Ident {
	syn::Ident::new(format!("\"{}\"", s))
}

fn get_options(attrs: &[syn::Attribute], name: &str) -> Result<Vec<syn::MetaItem>> {
	let options = attrs.iter().find(|a| a.name() == name).map(|a| &a.value);
	match options {
		Some(&syn::MetaItem::List(_, ref options)) => {
			options.iter().map(|o| match *o {
				syn::NestedMetaItem::MetaItem(ref m) => Ok(m.clone()),
				syn::NestedMetaItem::Literal(ref lit) => Err(format!("Unexpected meta item {:?}", lit).into())
			}).collect::<Result<Vec<_>>>()
		},
		Some(e) => Err(format!("Unexpected meta item {:?}", e).into()),
		None => Ok(vec![]),
	}
}

fn get_option<'a>(options: &'a [syn::MetaItem], name: &str) -> Result<&'a str> {
	let item = options.iter().find(|a| a.name() == name).chain_err(|| format!("Expected to find option {}", name))?;
	str_value_of_meta_item(item, name)
}

fn str_value_of_meta_item<'a>(item: &'a syn::MetaItem, name: &str) -> Result<&'a str> {
    match *item {
        syn::MetaItem::NameValue(_, syn::Lit::Str(ref value, _)) => Ok(&*value),
        _ => Err(format!(r#"`{}` must be in the form `#[{}="something"]`"#, name, name).into()),
    }
}

fn normalize_path(relative_path: &str) -> Result<PathBuf> {
	// workaround for https://github.com/rust-lang/rust/issues/43860
	let cargo_toml_directory = env::var("CARGO_MANIFEST_DIR").chain_err(|| "Cannot find manifest file")?;
	let mut path: PathBuf = cargo_toml_directory.into();
	path.push(relative_path);
	Ok(path)
}

fn impl_contract_function(function: &Function) -> quote::Tokens {
	let query_name = string_ident(&function.name);
	let name = syn::Ident::new(function.name.to_snake_case());
	let function_name = syn::Ident::new(function.name.to_camel_case());

	quote! {
		pub fn #name(&self) -> functions::#function_name {
			functions::#function_name::from_x(self.contract.function(#query_name).expect(INTERNAL_ERR))
		}
	}
}

fn rust_type(input: &ParamType) -> syn::Ident {
	match *input {
		ParamType::Address => "ethabi::Address".into(),
		ParamType::Bytes => "ethabi::Bytes".into(),
		ParamType::FixedBytes(32) => "ethabi::Hash".into(),
		ParamType::FixedBytes(size) => format!("[u8; {}]", size).into(),
		ParamType::Int(_) => "ethabi::Int".into(),
		ParamType::Uint(_) => "ethabi::Uint".into(),
		ParamType::Bool => "bool".into(),
		ParamType::String => "String".into(),
		ParamType::Array(ref kind) => format!("Vec<{}>", rust_type(&*kind)).into(),
		ParamType::FixedArray(ref kind, size) => format!("[{}; {}]", rust_type(&*kind), size).into(),
	}
}

fn template_param_type(input: &ParamType, index: usize) -> syn::Ident {
	match *input {
		ParamType::Address => format!("T{}: Into<ethabi::Address>", index).into(),
		ParamType::Bytes => format!("T{}: Into<ethabi::Bytes>", index).into(),
		ParamType::FixedBytes(32) => format!("T{}: Into<ethabi::Hash>", index).into(),
		ParamType::FixedBytes(size) => format!("T{}: Into<[u8; {}]>", index, size).into(),
		ParamType::Int(_) => format!("T{}: Into<ethabi::Int>", index).into(),
		ParamType::Uint(_) => format!("T{}: Into<ethabi::Uint>", index).into(),
		ParamType::Bool => format!("T{}: Into<bool>", index).into(),
		ParamType::String => format!("T{}: Into<String>", index).into(),
		ParamType::Array(ref kind) => format!("T{}: IntoIterator<Item = U{}>, U{}: Into<{}>", index, index, index, rust_type(&*kind)).into(),
		ParamType::FixedArray(ref kind, size) => format!("T{}: Into<[U{}; {}]>, U{}: Into<{}>", index, index, size, index, rust_type(&*kind)).into(),
	}
}

fn from_template_param(input: &ParamType, name: &syn::Ident) -> syn::Ident {
	match *input {
		ParamType::Array(_) => format!("{}.into_iter().map(Into::into).collect::<Vec<_>>()", name).into(),
		ParamType::FixedArray(_, _) => format!("(Box::new({}.into()) as Box<[_]>).into_vec().into_iter().map(Into::into).collect::<Vec<_>>()", name).into(),
		_ => format!("{}.into()", name).into(),
	}
}

fn to_token(name: &syn::Ident, kind: &ParamType) -> quote::Tokens {
	match *kind {
		ParamType::Address => quote! { ethabi::Token::Address(#name) },
		ParamType::Bytes => quote! { ethabi::Token::Bytes(#name) },
		ParamType::FixedBytes(_) => quote! { ethabi::Token::FixedBytes(#name.to_vec()) },
		ParamType::Int(_) => quote! { ethabi::Token::Int(#name) },
		ParamType::Uint(_) => quote! { ethabi::Token::Uint(#name) },
		ParamType::Bool => quote! { ethabi::Token::Bool(#name) },
		ParamType::String => quote! { ethabi::Token::String(#name) },
		ParamType::Array(ref kind) => {
			let inner_name: syn::Ident = "inner".into();
			let inner_loop = to_token(&inner_name, kind);
			quote! {
				// note the double {{
				{
					let v = #name.into_iter().map(|#inner_name| #inner_loop).collect();
					ethabi::Token::Array(v)
				}
			}
		}
		ParamType::FixedArray(ref kind, _) => {
			let inner_name: syn::Ident = "inner".into();
			let inner_loop = to_token(&inner_name, kind);
			quote! {
				// note the double {{
				{
					let v = #name.into_iter().map(|#inner_name| #inner_loop).collect();
					ethabi::Token::FixedArray(v)
				}
			}
		},
	}
}

fn from_token(kind: &ParamType, token: &syn::Ident) -> quote::Tokens {
	match *kind {
		ParamType::Address => quote! { #token.to_address().expect(super::INTERNAL_ERR) },
		ParamType::Bytes => quote! { #token.to_bytes().expect(super::INTERNAL_ERR) },
		ParamType::FixedBytes(size) => {
			let size: syn::Ident = format!("{}", size).into();
			quote! {
				{
					let mut result = [0u8; #size];
					let v = #token.to_fixed_bytes().expect(super::INTERNAL_ERR);
					result.copy_from_slice(&v);
					result
				}
			}
		},
		ParamType::Int(_) => quote! { #token.to_int().expect(super::INTERNAL_ERR) },
		ParamType::Uint(_) => quote! { #token.to_uint().expect(super::INTERNAL_ERR) },
		ParamType::Bool => quote! { #token.to_bool().expect(super::INTERNAL_ERR) },
		ParamType::String => quote! { #token.to_string().expect(super::INTERNAL_ERR) },
		ParamType::Array(ref kind) => {
			let inner: syn::Ident = "inner".into();
			let inner_loop = from_token(kind, &inner);
			quote! {
				#token.to_array().expect(super::INTERNAL_ERR).into_iter()
					.map(|#inner| #inner_loop)
					.collect()
			}
		},
		ParamType::FixedArray(ref kind, size) => {
			let inner: syn::Ident = "inner".into();
			let inner_loop = from_token(kind, &inner);
			let to_array = vec![quote! { iter.next() }; size];
			quote! {
				{
					let iter = #token.to_array().expect(super::INTERNAL_ERR).into_iter()
						.map(|#inner| #inner_loop);
					[#(#to_array),*]
				}
			}
		},
	}
}

fn impl_contract_event(event: &Event) -> quote::Tokens {
	let query_name = string_ident(&event.name);
	let name = syn::Ident::new(event.name.to_snake_case());
	let event_name = syn::Ident::new(event.name.to_camel_case());
	quote! {
		pub fn #name(&self) -> events::#event_name {
			events::#event_name::from_x(self.contract.event(#query_name).expect(INTERNAL_ERR))
		}
	}
}

fn impl_contract_constructor(constructor: &Constructor) -> quote::Tokens {
	// [param0, hello_world, param2]
	let names: Vec<_> = constructor.inputs
		.iter()
		.enumerate()
		.map(|(index, param)| if param.name.is_empty() {
			syn::Ident::new(format!("param{}", index))
		} else {
			param.name.to_snake_case().into()
		}).collect();

	// [Uint, Bytes, Vec<Uint>]
	let kinds: Vec<_> = constructor.inputs
		.iter()
		.map(|param| rust_type(&param.kind))
		.collect();

	// [T0, T1, T2]
	let template_names: Vec<_> = kinds.iter().enumerate()
		.map(|(index, _)| syn::Ident::new(format!("T{}", index)))
		.collect();

	// [T0: Into<Uint>, T1: Into<Bytes>, T2: IntoIterator<Item = U2>, U2 = Into<Uint>]
	let template_params: Vec<_> = constructor.inputs.iter().enumerate()
		.map(|(index, param)| template_param_type(&param.kind, index))
		.collect();

	// [param0: T0, hello_world: T1, param2: T2]
	let params: Vec<_> = names.iter().zip(template_names.iter())
		.map(|(param_name, template_name)| quote! { #param_name: #template_name })
		.collect();

	// [Token::Uint(param0.into()), Token::Bytes(hello_world.into()), Token::Array(param2.into())]
	let usage: Vec<_> = names.iter().zip(constructor.inputs.iter())
		.map(|(param_name, param)| to_token(&from_template_param(&param.kind, param_name), &param.kind))
		.collect();

	quote! {
		pub fn constructor<#(#template_params),*>(&self, code: ethabi::Bytes, #(#params),* ) -> ethabi::Bytes {
			let v: Vec<ethabi::Token> = vec![#(#usage),*];
			self.contract.constructor
				.as_ref()
				.expect(INTERNAL_ERR)
				.encode_input(code, &v)
				.expect("encode_input not to fail; ethabi_derive bug")
		}
	}
}

fn declare_logs(event: &Event) -> quote::Tokens {
	let name = syn::Ident::new(event.name.to_camel_case());
	let names: Vec<_> = event.inputs
		.iter()
		.enumerate()
		.map(|(index, param)| if param.name.is_empty() {
			syn::Ident::new(format!("param{}", index))
		} else {
			param.name.to_snake_case().into()
		}).collect();
	let kinds: Vec<_> = event.inputs
		.iter()
		.map(|param| rust_type(&param.kind))
		.collect();
	let params: Vec<_> = names.iter().zip(kinds.iter())
		.map(|(param_name, kind)| quote! { pub #param_name: #kind, })
		.collect();

	quote! {
		pub struct #name {
			#(#params)*
		}
	}
}

fn declare_events(event: &Event) -> quote::Tokens {
	let name = syn::Ident::new(event.name.to_camel_case());

	// parse log

	let names: Vec<_> = event.inputs
		.iter()
		.enumerate()
		.map(|(index, param)| if param.name.is_empty() {
			if param.indexed {
				syn::Ident::new(format!("topic{}", index))
			} else {
				syn::Ident::new(format!("param{}", index))
			}
		} else {
			param.name.to_snake_case().into()
		}).collect();

	let log_iter = syn::Ident::new("log.next().expect(super::INTERNAL_ERR).value");

	let to_log: Vec<_> = event.inputs
		.iter()
		.map(|param| from_token(&param.kind, &log_iter))
		.collect();

	let log_params: Vec<_> = names.iter().zip(to_log.iter())
		.map(|(param_name, convert)| quote! { #param_name: #convert })
		.collect();

	// create filter

	let topic_names: Vec<_> = event.inputs
		.iter()
		.enumerate()
		.filter(|&(_, param)| param.indexed)
		.map(|(index, param)| if param.name.is_empty() {
			syn::Ident::new(format!("topic{}", index))
		} else {
			param.name.to_snake_case().into()
		})
		.collect();

	let topic_kinds: Vec<_> = event.inputs
		.iter()
		.filter(|param| param.indexed)
		.map(|param| rust_type(&param.kind))
		.collect();

	// [T0, T1, T2]
	let template_names: Vec<_> = topic_kinds.iter().enumerate()
		.map(|(index, _)| syn::Ident::new(format!("T{}", index)))
		.collect();

	let params: Vec<_> = topic_names.iter().zip(template_names.iter())
		.map(|(param_name, template_name)| quote! { #param_name: #template_name })
		.collect();

	let template_params: Vec<_> = topic_kinds.iter().zip(template_names.iter())
		.map(|(kind, template_name)| quote! { #template_name: Into<ethabi::Topic<#kind>> })
		.collect();

	let to_filter: Vec<_> = topic_names.iter().zip(event.inputs.iter().filter(|p| p.indexed))
		.enumerate()
		.take(3)
		.map(|(index, (param_name, param))| {
			let topic = syn::Ident::new(format!("topic{}", index));
			let i = "i".into();
			let to_token = to_token(&i, &param.kind);
			quote! { #topic: #param_name.into().map(|#i| #to_token), }
		})
		.collect();

	quote! {
		pub struct #name<'a> {
			event: &'a ethabi::Event,
		}

		impl<'a> #name<'a> {
			pub(super) fn from_x(event: &'a ethabi::Event) -> Self {
				#name {
					event,
				}
			}

			/// Parses log.
			pub fn parse_log(&self, log: ethabi::RawLog) -> ethabi::Result<super::logs::#name> {
				let mut log = self.event.parse_log(log)?.params.into_iter();
				let result = super::logs::#name {
					#(#log_params),*
				};
				Ok(result)
			}

			/// Creates topic filter.
			pub fn create_filter<#(#template_params),*>(&self, #(#params),*) -> ethabi::TopicFilter {
				let raw = ethabi::RawTopicFilter {
					#(#to_filter)*
					..Default::default()
				};

				self.event.create_filter(raw).expect(super::INTERNAL_ERR)
			}
		}
	}
}

fn declare_functions(function: &Function) -> quote::Tokens {
	let name = syn::Ident::new(function.name.to_camel_case());

	// [param0, hello_world, param2]
	let names: Vec<_> = function.inputs
		.iter()
		.enumerate()
		.map(|(index, param)| if param.name.is_empty() {
			syn::Ident::new(format!("param{}", index))
		} else {
			param.name.to_snake_case().into()
		}).collect();

	// [Uint, Bytes, Vec<Uint>]
	let kinds: Vec<_> = function.inputs
		.iter()
		.map(|param| rust_type(&param.kind))
		.collect();

	// [T0, T1, T2]
	let template_names: Vec<_> = kinds.iter().enumerate()
		.map(|(index, _)| syn::Ident::new(format!("T{}", index)))
		.collect();

	// [T0: Into<Uint>, T1: Into<Bytes>, T2: IntoIterator<Item = U2>, U2 = Into<Uint>]
	let template_params: Vec<_> = function.inputs.iter().enumerate()
		.map(|(index, param)| template_param_type(&param.kind, index))
		.collect();

	// [param0: T0, hello_world: T1, param2: T2]
	let params: Vec<_> = names.iter().zip(template_names.iter())
		.map(|(param_name, template_name)| quote! { #param_name: #template_name })
		.collect();

	// [Token::Uint(param0.into()), Token::Bytes(hello_world.into()), Token::Array(param2.into_iter().map(Into::into).collect())]
	let usage: Vec<_> = names.iter().zip(function.inputs.iter())
		.map(|(param_name, param)| to_token(&from_template_param(&param.kind, param_name), &param.kind))
		.collect();

	let output_impl = if !function.constant {
		quote! {}
	} else {
		let output_kinds = match function.outputs.len() {
			0 => quote! {()},
			1 => {
				let t = rust_type(&function.outputs[0].kind);
				quote! { #t }
			},
			_ => {
				let outs: Vec<_> = function.outputs
					.iter()
					.map(|param| rust_type(&param.kind))
					.collect();
				quote! { (#(#outs),*) }
			}
		};

		let o_impl = match function.outputs.len() {
			0 => quote! { Ok(()) },
			1 => {
				let o = "out".into();
				let from_first = from_token(&function.outputs[0].kind, &o);
				quote! {
					let out = self.function.decode_output(output)?.into_iter().next().expect(super::INTERNAL_ERR);
					Ok(#from_first)
				}
			},
			_ => {
				let o = "out.into_iter().next().expect(super::INTERNAL_ERR)".into();
				let outs: Vec<_> = function.outputs
					.iter()
					.map(|param| from_token(&param.kind, &o))
					.collect();

				quote! {
					let out = self.function.decode_output(output)?.into_iter();
					Ok(( #(#outs),* ))
				}
			},
		};

		quote! {
			pub fn output(&self, output: &[u8]) -> ethabi::Result<#output_kinds> {
				#o_impl
			}
		}
	};

	quote! {
		pub struct #name<'a> {
			function: &'a ethabi::Function,
		}

		impl<'a> #name<'a> {
			pub(super) fn from_x(function: &'a ethabi::Function) -> Self {
				#name {
					function,
				}
			}

			pub fn input<#(#template_params),*>(&self, #(#params),*) -> ethabi::Bytes {
				let v: Vec<ethabi::Token> = vec![#(#usage),*];
				self.function.encode_input(&v).expect("encode_input not to fail; ethabi_derive bug")
			}

			#output_impl
		}
	}
}
