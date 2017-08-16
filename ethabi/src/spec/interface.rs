//! Contract interface.

use std::io;
use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Deserializer};
use serde::de::{Visitor, SeqAccess};
use serde_json;
use errors::Error;
use super::{Operation, Constructor, Event};
use {Function};

/// Contract interface.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Interface {
	/// Contract constructor.
	pub constructor: Option<Constructor>,
	/// Contract functions.
	pub functions: HashMap<String, Function>,
	/// Contract events.
	pub events: HashMap<String, Event>,
	/// Contract has fallback function.
	pub fallback: bool,
}

impl<'a> Deserialize<'a> for Interface {
	fn deserialize<D>(deserializer: D) -> Result<Interface, D::Error> where D: Deserializer<'a> {
		deserializer.deserialize_any(InterfaceVisitor)
	}
}

struct InterfaceVisitor;

impl<'a> Visitor<'a> for InterfaceVisitor {
	type Value = Interface;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("valid abi spec file")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'a> {
		let mut result = Interface::default();

		while let Some(operation) = seq.next_element()? {
			match operation {
				Operation::Constructor(constructor) => {
					result.constructor = Some(constructor);
				},
				Operation::Function(func) => {
					result.functions.insert(func.name.clone(), func);
				},
				Operation::Event(event) => {
					result.events.insert(event.name.clone(), event);
				},
				Operation::Fallback => {
					result.fallback = true;
				},
			}
		}

		Ok(result)
	}
}

impl Interface {
	/// Loads interface from json.
	pub fn load<T: io::Read>(reader: T) -> Result<Self, Error> {
		serde_json::from_reader(reader).map_err(From::from)
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::Interface;

	#[test]
	fn deserialize_interface() {
		let s = r#"[{
			"type":"event",
			"inputs": [{
				"name":"a",
				"type":"uint256",
				"indexed":true
			},{
				"name":"b",
				"type":"bytes32",
				"indexed":false
			}],
			"name":"Event2",
			"anonymous": false
		}, {
			"type":"function",
			"inputs": [{
				"name":"a",
				"type":"uint256"
			}],
			"name":"foo",
			"outputs": []
		}]"#;

		let _: Interface = serde_json::from_str(s).unwrap();
	}

	#[test]
	fn deserialize_event2() {
		let s = r#"[{
			"inputs": [{
				"name": "_curator",
				"type": "address"
			}, {
				"name": "_daoCreator",
				"type": "address"
			}, {
				"name": "_proposalDeposit",
				"type": "uint256"
			}, {
				"name": "_minTokensToCreate",
				"type": "uint256"
			}, {
				"name": "_closingTime",
				"type": "uint256"
			}, {
				"name": "_privateCreation",
				"type": "address"
			}],
			"type": "constructor"
		}]"#;

		let _: Interface = serde_json::from_str(s).unwrap();

	}
}
