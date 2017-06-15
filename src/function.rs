//! Contract function call builder.

use spec::{Function as FunctionInterface, ParamType};
use token::Token;
use encoder::Encoder;
use decoder::Decoder;
use signature::signature;
use error::Error;

/// Contract function call builder.
#[derive(Clone, Debug)]
pub struct Function {
	interface: FunctionInterface,
}

impl Function {
	/// Creates new function call builder.
	pub fn new(interface: FunctionInterface) -> Self {
		Function {
			interface: interface
		}
	}

	/// Returns function params.
	pub fn input_params(&self) -> Vec<ParamType> {
		self.interface.input_param_types()
	}

	/// Return output params.
	pub fn output_params(&self) -> Vec<ParamType> {
		self.interface.output_param_types()
	}

	/// Prepares ABI function call with given input params.
	pub fn encode_call(&self, tokens: Vec<Token>) -> Result<Vec<u8>, Error> {
		let params = self.interface.input_param_types();

		if !type_check(&tokens, &params) {
			return Err(Error::InvalidData);
		}

		let signed = signature(&self.interface.name, &params);
		let encoded = Encoder::encode(tokens);
		Ok(signed.into_iter().chain(encoded.into_iter()).collect())
	}

	/// Parses the ABI function output to list of tokens.
	pub fn decode_output(&self, data: Vec<u8>) -> Result<Vec<Token>, Error> {
		Decoder::decode(&self.interface.output_param_types(), data)
	}

	/// Get the name of the function.
	pub fn name(&self) -> &str {
		&self.interface.name
	}
}

/// Check if all the types of the tokens match the given parameter types.
pub fn type_check(tokens: &[Token], param_types: &[ParamType]) -> bool {
	param_types.len() == tokens.len() && {
		param_types.iter().zip(tokens).all(|e| {
			let (param_type, token) = e;
			token.type_check(param_type)
		})
	}
}

#[cfg(test)]
mod tests {
	use hex::FromHex;
	use spec::{Function as FunctionInterface, ParamType, Param};
	use token::Token;
	use super::Function;

	#[test]
	fn test_function_encode_call() {
		let interface = FunctionInterface {
			name: "baz".to_owned(),
			inputs: vec![Param {
				name: "a".to_owned(),
				kind: ParamType::Uint(32),
			}, Param {
				name: "b".to_owned(),
				kind: ParamType::Bool,
			}],
			outputs: vec![]
		};

		let func = Function::new(interface);
		let mut uint = [0u8; 32];
		uint[31] = 69;
		let encoded = func.encode_call(vec![Token::Uint(uint), Token::Bool(true)]).unwrap();
		let expected = "cdcd77c000000000000000000000000000000000000000000000000000000000000000450000000000000000000000000000000000000000000000000000000000000001".from_hex().unwrap();
		assert_eq!(encoded, expected);
	}
}
