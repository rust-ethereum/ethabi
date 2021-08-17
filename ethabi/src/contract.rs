// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{errors, operation::Operation, Constructor, Error, Event, Function};
use serde::{
	de::{SeqAccess, Visitor},
	ser::SerializeSeq,
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
	collections::{hash_map::Values, HashMap},
	fmt, io,
	iter::Flatten,
};

/// API building calls to contracts ABI.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contract {
	/// Contract constructor.
	pub constructor: Option<Constructor>,
	/// Contract functions.
	pub functions: HashMap<String, Vec<Function>>,
	/// Contract events, maps signature to event.
	pub events: HashMap<String, Vec<Event>>,
	/// Contract has receive function.
	pub receive: bool,
	/// Contract has fallback function.
	pub fallback: bool,
}

impl<'a> Deserialize<'a> for Contract {
	fn deserialize<D>(deserializer: D) -> Result<Contract, D::Error>
	where
		D: Deserializer<'a>,
	{
		deserializer.deserialize_any(ContractVisitor)
	}
}

struct ContractVisitor;

impl<'a> Visitor<'a> for ContractVisitor {
	type Value = Contract;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("valid abi spec file")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'a>,
	{
		let mut result = Contract::default();
		while let Some(operation) = seq.next_element()? {
			match operation {
				Operation::Constructor(constructor) => {
					result.constructor = Some(constructor);
				}
				Operation::Function(func) => {
					result.functions.entry(func.name.clone()).or_default().push(func);
				}
				Operation::Event(event) => {
					result.events.entry(event.name.clone()).or_default().push(event);
				}
				Operation::Fallback => {
					result.fallback = true;
				}
				Operation::Receive => {
					result.receive = true;
				}
			}
		}

		Ok(result)
	}
}

impl Serialize for Contract {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		// Serde's FlatMapSerializer is private, so we'll have to improvise...
		#[derive(Serialize)]
		#[serde(tag = "type")]
		enum OperationRef<'a> {
			#[serde(rename = "constructor")]
			Constructor(&'a Constructor),

			#[serde(rename = "function")]
			Function(&'a Function),

			#[serde(rename = "event")]
			Event(&'a Event),

			#[serde(rename = "fallback")]
			Fallback,

			#[serde(rename = "receive")]
			Receive,
		}

		let mut seq = serializer.serialize_seq(None)?;

		if let Some(constructor) = &self.constructor {
			seq.serialize_element(&OperationRef::Constructor(constructor))?;
		}

		for functions in self.functions.values() {
			for function in functions {
				seq.serialize_element(&OperationRef::Function(function))?;
			}
		}

		for events in self.events.values() {
			for event in events {
				seq.serialize_element(&OperationRef::Event(event))?;
			}
		}

		if self.receive {
			seq.serialize_element(&OperationRef::Receive)?;
		}

		if self.fallback {
			seq.serialize_element(&OperationRef::Fallback)?;
		}

		seq.end()
	}
}

impl Contract {
	/// Loads contract from json.
	pub fn load<T: io::Read>(reader: T) -> errors::Result<Self> {
		serde_json::from_reader(reader).map_err(From::from)
	}

	/// Creates constructor call builder.
	pub fn constructor(&self) -> Option<&Constructor> {
		self.constructor.as_ref()
	}

	/// Get the function named `name`, the first if there are overloaded
	/// versions of the same function.
	pub fn function(&self, name: &str) -> errors::Result<&Function> {
		self.functions.get(name).into_iter().flatten().next().ok_or_else(|| Error::InvalidName(name.to_owned()))
	}

	/// Get the contract event named `name`, the first if there are multiple.
	pub fn event(&self, name: &str) -> errors::Result<&Event> {
		self.events.get(name).into_iter().flatten().next().ok_or_else(|| Error::InvalidName(name.to_owned()))
	}

	/// Get all contract events named `name`.
	pub fn events_by_name(&self, name: &str) -> errors::Result<&Vec<Event>> {
		self.events.get(name).ok_or_else(|| Error::InvalidName(name.to_owned()))
	}

	/// Get all functions named `name`.
	pub fn functions_by_name(&self, name: &str) -> errors::Result<&Vec<Function>> {
		self.functions.get(name).ok_or_else(|| Error::InvalidName(name.to_owned()))
	}

	/// Iterate over all functions of the contract in arbitrary order.
	pub fn functions(&self) -> Functions {
		Functions(self.functions.values().flatten())
	}

	/// Iterate over all events of the contract in arbitrary order.
	pub fn events(&self) -> Events {
		Events(self.events.values().flatten())
	}
}

/// Contract functions iterator.
pub struct Functions<'a>(Flatten<Values<'a, String, Vec<Function>>>);

impl<'a> Iterator for Functions<'a> {
	type Item = &'a Function;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

/// Contract events iterator.
pub struct Events<'a>(Flatten<Values<'a, String, Vec<Event>>>);

impl<'a> Iterator for Events<'a> {
	type Item = &'a Event;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
	use crate::{tests::assert_ser_de, Constructor, Contract, Event, EventParam, Function, Param, ParamType};
	use std::{collections::HashMap, iter::FromIterator};

	#[test]
	fn empty() {
		let json = "[]";

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: HashMap::new(),
				events: HashMap::new(),
				receive: false,
				fallback: false,
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn constructor() {
		let json = r#"
			[
				{
					"type": "constructor",
					"inputs": [
						{
							"name":"a",
							"type":"address"
						}
					]
				}
			]
		"#;

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: Some(Constructor {
					inputs: vec![Param { name: "a".to_string(), kind: ParamType::Address, internal_type: None }]
				}),
				functions: HashMap::new(),
				events: HashMap::new(),
				receive: false,
				fallback: false,
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn functions() {
		let json = r#"
			[
				{
					"type": "function",
					"name": "foo",
					"inputs": [
						{
							"name":"a",
							"type":"address"
						}
					],
					"outputs": [
						{
							"name": "res",
							"type":"address"
						}
					]
				},
				{
					"type": "function",
					"name": "bar",
					"inputs": [],
					"outputs": []
				}
			]
		"#;

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: HashMap::from_iter(vec![
					(
						"foo".to_string(),
						vec![Function {
							name: "foo".to_string(),
							inputs: vec![Param {
								name: "a".to_string(),
								kind: ParamType::Address,
								internal_type: None
							}],
							outputs: vec![Param {
								name: "res".to_string(),
								kind: ParamType::Address,
								internal_type: None
							}],
							constant: false,
							state_mutability: Default::default()
						}]
					),
					(
						"bar".to_string(),
						vec![Function {
							name: "bar".to_string(),
							inputs: vec![],
							outputs: vec![],
							constant: false,
							state_mutability: Default::default()
						}]
					)
				]),
				events: HashMap::new(),
				receive: false,
				fallback: false,
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn functions_overloads() {
		let json = r#"
			[
				{
					"type": "function",
					"name": "foo",
					"inputs": [
						{
							"name":"a",
							"type":"address"
						}
					],
					"outputs": [
						{
							"name": "res",
							"type":"address"
						}
					]
				},
				{
					"type": "function",
					"name": "foo",
					"inputs": [],
					"outputs": []
				}
			]
		"#;

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: HashMap::from_iter(vec![(
					"foo".to_string(),
					vec![
						Function {
							name: "foo".to_string(),
							inputs: vec![Param {
								name: "a".to_string(),
								kind: ParamType::Address,
								internal_type: None
							}],
							outputs: vec![Param {
								name: "res".to_string(),
								kind: ParamType::Address,
								internal_type: None
							}],
							constant: false,
							state_mutability: Default::default()
						},
						Function {
							name: "foo".to_string(),
							inputs: vec![],
							outputs: vec![],
							constant: false,
							state_mutability: Default::default()
						}
					]
				)]),
				events: HashMap::new(),
				receive: false,
				fallback: false,
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn events() {
		let json = r#"
			[
				{
					"type": "event",
					"name": "foo",
					"inputs": [
						{
							"name":"a",
							"type":"address"
						}
					],
					"anonymous": false
				},
				{
					"type": "event",
					"name": "bar",
					"inputs": [
						{
							"name":"a",
							"type":"address",
							"indexed": true
						}
					],
					"anonymous": false
				}
			]
		"#;

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: HashMap::new(),
				events: HashMap::from_iter(vec![
					(
						"foo".to_string(),
						vec![Event {
							name: "foo".to_string(),
							inputs: vec![EventParam {
								name: "a".to_string(),
								kind: ParamType::Address,
								indexed: false
							}],
							anonymous: false
						}]
					),
					(
						"bar".to_string(),
						vec![Event {
							name: "bar".to_string(),
							inputs: vec![EventParam { name: "a".to_string(), kind: ParamType::Address, indexed: true }],
							anonymous: false
						}]
					)
				]),
				receive: false,
				fallback: false,
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn events_overload() {
		let json = r#"
			[
				{
					"type": "event",
					"name": "foo",
					"inputs": [
						{
							"name":"a",
							"type":"address"
						}
					],
					"anonymous": false
				},
				{
					"type": "event",
					"name": "foo",
					"inputs": [
						{
							"name":"a",
							"type":"address",
							"indexed": true
						}
					],
					"anonymous": false
				}
			]
		"#;

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: HashMap::new(),
				events: HashMap::from_iter(vec![(
					"foo".to_string(),
					vec![
						Event {
							name: "foo".to_string(),
							inputs: vec![EventParam {
								name: "a".to_string(),
								kind: ParamType::Address,
								indexed: false
							}],
							anonymous: false
						},
						Event {
							name: "foo".to_string(),
							inputs: vec![EventParam { name: "a".to_string(), kind: ParamType::Address, indexed: true }],
							anonymous: false
						}
					]
				)]),
				receive: false,
				fallback: false,
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn receive() {
		let json = r#"
			[
				{ "type": "receive" }
			]
		"#;

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: HashMap::new(),
				events: HashMap::new(),
				receive: true,
				fallback: false,
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn fallback() {
		let json = r#"
			[
				{ "type": "fallback" }
			]
		"#;

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: HashMap::new(),
				events: HashMap::new(),
				receive: false,
				fallback: true,
			}
		);

		assert_ser_de(&deserialized);
	}
}
