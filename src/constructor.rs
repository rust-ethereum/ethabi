//! Contract constructor call builder.

use spec::Constructor as ConstructorInterface;
use function::type_check;
use token::Token;
use error::Error;
use encoder::Encoder;

/// Contract constructor call builder.
#[derive(Clone, Debug)]
pub struct Constructor {
	_interface: ConstructorInterface,
}

impl Constructor {
	/// Creates new constructor call builder.
	pub fn new(interface: ConstructorInterface) -> Self {
		Constructor {
			_interface: interface
		}
	}

	/// Prepares ABI constructor call with given input params.
	pub fn encode_call(&self, tokens: Vec<Token>) -> Result<Vec<u8>, Error> {
		let params = self._interface.param_types();

		if type_check(&tokens, &params) {
			Ok(Encoder::encode(tokens))
		} else {
			Err(Error::InvalidData)
		}
	}
}
