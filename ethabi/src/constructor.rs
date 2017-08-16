//! Contract constructor call builder.
use {Param, Result, ErrorKind, Token, ParamType, encode};

/// Contract constructor specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Constructor {
	/// Constructor input.
	pub inputs: Vec<Param>,
}

impl Constructor {
	/// Returns all input params of given constructor.
	fn param_types(&self) -> Vec<ParamType> {
		self.inputs.iter()
			.map(|p| p.kind.clone())
			.collect()
	}

	/// Prepares ABI constructor call with given input params.
	pub fn encode_call(&self, tokens: &[Token]) -> Result<Vec<u8>> {
		let params = self.param_types();

		if Token::types_check(tokens, &params) {
			Ok(encode(tokens))
		} else {
			Err(ErrorKind::InvalidData.into())
		}
	}
}
