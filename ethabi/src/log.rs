use {Hash, Token};

/// Ethereum log.
#[derive(Debug, PartialEq)]
pub struct RawLog {
	/// Indexed event params are represented as log topics.
	pub topics: Vec<Hash>,
	/// Others are just plain data.
	pub data: Vec<u8>,
}

/// Decoded log param.
#[derive(Debug, PartialEq)]
pub struct LogParam {
	/// Decoded log name.
	pub name: String,
	/// Decoded log value.
	pub value: Token,
}

/// Decoded log.
#[derive(Debug, PartialEq)]
pub struct Log {
	/// Log params.
	pub params: Vec<LogParam>,
}
