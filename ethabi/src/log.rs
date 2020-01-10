use crate::{Hash, Token, Bytes, Result, TopicFilter};

/// Common filtering functions that are available for any event.
pub trait LogFilter {
	/// Match any log parameters.
	fn wildcard_filter(&self) -> TopicFilter;
}

/// trait common to things (events) that have an associated `Log` type
/// that can be parsed from a `RawLog`
pub trait ParseLog {
	/// the associated `Log` type that can be parsed from a `RawLog`
	/// by calling `parse_log`
	type Log;

	/// parse the associated `Log` type from a `RawLog`
	fn parse_log(&self, log: RawLog) -> Result<Self::Log>;
}

/// Ethereum log.
#[derive(Debug, PartialEq, Clone)]
pub struct RawLog {
	/// Indexed event params are represented as log topics.
	pub topics: Vec<Hash>,
	/// Others are just plain data.
	pub data: Bytes,
}

impl From<(Vec<Hash>, Bytes)> for RawLog {
	fn from(raw: (Vec<Hash>, Bytes)) -> Self {
		RawLog {
			topics: raw.0,
			data: raw.1,
		}
	}
}

/// Decoded log param.
#[derive(Debug, PartialEq, Clone)]
pub struct LogParam {
	/// Decoded log name.
	pub name: String,
	/// Decoded log value.
	pub value: Token,
}

/// Decoded log.
#[derive(Debug, PartialEq, Clone)]
pub struct Log {
	/// Log params.
	pub params: Vec<LogParam>,
}
