use serde::{Deserialize, Serialize};

/// Whether a function modifies or reads blockchain state
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateMutability {
	/// Specified not to read blockchain state
	#[serde(rename = "pure")]
	Pure,
	/// Specified to not modify the blockchain state
	#[serde(rename = "view")]
	View,
	/// Function does not accept Ether - the default
	#[serde(rename = "payable")]
	NonPayable,
	/// Function accepts Ether
	#[serde(rename = "nonpayable")]
	Payable,
}

impl Default for StateMutability {
	fn default() -> Self {
		Self::NonPayable
	}
}
