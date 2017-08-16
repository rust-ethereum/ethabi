//! Contract function call builder.

use spec::{Param, ParamType};
use signature::short_signature;
use {Token, Result, ErrorKind, Encoder, Bytes, Decoder};

/// Contract function specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Function {
	/// Function name.
	pub name: String,
	/// Function input.
	pub inputs: Vec<Param>,
	/// Function output.
	pub outputs: Vec<Param>,
}

impl Function {
	/// Returns all input params of given function.
	fn input_param_types(&self) -> Vec<ParamType> {
		self.inputs.iter()
			.map(|p| p.kind.clone())
			.collect()
	}

	/// Returns all output params of given function.
	fn output_param_types(&self) -> Vec<ParamType> {
		self.outputs.iter()
			.map(|p| p.kind.clone())
			.collect()
	}

	/// Prepares ABI function call with given input params.
	pub fn encode_input(&self, tokens: Vec<Token>) -> Result<Bytes> {
		let params = self.input_param_types();

		if !Token::types_check(&tokens, &params) {
			return Err(ErrorKind::InvalidData.into());
		}

		let signed = short_signature(&self.name, &params).to_vec();
		let encoded = Encoder::encode(tokens);
		Ok(signed.into_iter().chain(encoded.into_iter()).collect())
	}

	/// Parses the ABI function output to list of tokens.
	pub fn decode_output(&self, data: Bytes) -> Result<Vec<Token>> {
		Decoder::decode(&self.output_param_types(), data)
	}
}

#[cfg(test)]
mod tests {
	use spec::{Param, ParamType};
	use hex::FromHex;
	use token::Token;
	use super::Function;

	#[test]
	fn test_function_encode_call() {
		let interface = Function {
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

		let func = Function::from(interface);
		let mut uint = [0u8; 32];
		uint[31] = 69;
		let encoded = func.encode_input(vec![Token::Uint(uint), Token::Bool(true)]).unwrap();
		let expected = "cdcd77c000000000000000000000000000000000000000000000000000000000000000450000000000000000000000000000000000000000000000000000000000000001".from_hex().unwrap();
		assert_eq!(encoded, expected);
	}
}

// Contract function call builder.
//#[derive(Clone, Debug, PartialEq)]
//pub struct Function {
	//interface: FunctionInterface,
//}

//impl From<FunctionInterface> for Function {
	//fn from(interface: FunctionInterface) -> Self {
		//Function {
			//interface,
		//}
	//}
//}

//impl Function {
	///// Returns function params.
	//pub fn input_params(&self) -> Vec<Param> {
		//self.interface.inputs.clone().into_iter().map(Into::into).collect()
	//}

	///// Return output params.
	//pub fn output_params(&self) -> Vec<ParamType> {
		//self.interface.output_param_types()
	//}


	///// Parses the ABI function output to list of tokens.
	//pub fn decode_output(&self, data: Vec<u8>) -> Result<Vec<Token>, Error> {
		//Decoder::decode(&self.interface.output_param_types(), data)
	//}

	///// Get the name of the function.
	//pub fn name(&self) -> &str {
		//&self.interface.name
	//}
//}

