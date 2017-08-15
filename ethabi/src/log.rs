use Hash;

/// Ethereum log.
pub struct Log {
	/// Indexed event params are represented as log topics.
	pub topics: Vec<Hash>,
	/// Others are just plain data.
	pub data: Vec<u8>,
}
