#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Whether a function modifies or reads blockchain state
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StateMutability {
	/// Specified not to read blockchain state
	#[cfg_attr(feature = "std", serde(rename = "pure"))]
	Pure,
	/// Specified to not modify the blockchain state
	#[cfg_attr(feature = "std", serde(rename = "view"))]
	View,
	/// Function does not accept Ether - the default
	#[cfg_attr(feature = "std", serde(rename = "nonpayable"))]
	NonPayable,
	/// Function accepts Ether
	#[cfg_attr(feature = "std", serde(rename = "payable"))]
	Payable,
}

impl Default for StateMutability {
	fn default() -> Self {
		Self::NonPayable
	}
}

#[cfg(test)]
mod test {
	use crate::{tests::assert_json_eq, StateMutability};

	#[test]
	fn state_mutability() {
		let json = r#"
			[
				"pure",
				"view",
				"nonpayable",
				"payable"
			]
		"#;

		let deserialized: Vec<StateMutability> = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			vec![StateMutability::Pure, StateMutability::View, StateMutability::NonPayable, StateMutability::Payable,]
		);

		assert_json_eq(json, &serde_json::to_string(&deserialized).unwrap());
	}
}
