use std::io::Error as IoError;
use hex::FromHexError;
use docopt;
use ethabi::spec::Error as SpecError;
use ethabi::spec::param_type::Error as ParamError;
use ethabi::token::Error as TokenizerError;
use ethabi::Error as DecoderError;

#[derive(Debug)]
pub enum Error {
	Io(IoError),
	Hex(FromHexError),
	Spec(SpecError),
	Param(ParamError),
	Tokenizer(TokenizerError),
	Decoder(DecoderError),
	Docopt(docopt::Error),
}

impl From<IoError> for Error {
	fn from(err: IoError) -> Self {
		Error::Io(err)
	}
}

impl From<FromHexError> for Error {
	fn from(err: FromHexError) -> Self {
		Error::Hex(err)
	}
}

impl From<SpecError> for Error {
	fn from(err: SpecError) -> Self {
		Error::Spec(err)
	}
}

impl From<ParamError> for Error {
	fn from(err: ParamError) -> Self {
		Error::Param(err)
	}
}

impl From<TokenizerError> for Error {
	fn from(err: TokenizerError) -> Self {
		Error::Tokenizer(err)
	}
}

impl From<DecoderError> for Error {
	fn from(err: DecoderError) -> Self {
		Error::Decoder(err)
	}
}

impl From<docopt::Error> for Error {
	fn from(err: docopt::Error) -> Self {
		Error::Docopt(err)
	}
}
