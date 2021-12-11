// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use alloc::collections::{btree_map::Values, BTreeMap};
#[cfg(feature = "full-serde")]
use core::fmt;
use core::iter::Flatten;
#[cfg(feature = "full-serde")]
use std::io;

#[cfg(feature = "full-serde")]
use serde::{
	de::{SeqAccess, Visitor},
	ser::SerializeSeq,
	Deserialize, Deserializer, Serialize, Serializer,
};

#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;
#[cfg(feature = "full-serde")]
use crate::operation::Operation;
use crate::{error::Error as AbiError, errors, Constructor, Error, Event, Function};

/// API building calls to contracts ABI.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contract {
	/// Contract constructor.
	pub constructor: Option<Constructor>,
	/// Contract functions.
	pub functions: BTreeMap<String, Vec<Function>>,
	/// Contract events, maps signature to event.
	pub events: BTreeMap<String, Vec<Event>>,
	/// Contract errors, maps signature to error.
	pub errors: BTreeMap<String, Vec<AbiError>>,
	/// Contract has receive function.
	pub receive: bool,
	/// Contract has fallback function.
	pub fallback: bool,
}

#[cfg(feature = "full-serde")]
impl<'a> Deserialize<'a> for Contract {
	fn deserialize<D>(deserializer: D) -> Result<Contract, D::Error>
	where
		D: Deserializer<'a>,
	{
		deserializer.deserialize_any(ContractVisitor)
	}
}

#[cfg(feature = "full-serde")]
struct ContractVisitor;

#[cfg(feature = "full-serde")]
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
				Operation::Error(error) => {
					result.errors.entry(error.name.clone()).or_default().push(error);
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

#[cfg(feature = "full-serde")]
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

			#[serde(rename = "error")]
			Error(&'a AbiError),

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

		for errors in self.errors.values() {
			for error in errors {
				seq.serialize_element(&OperationRef::Error(error))?;
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
	#[cfg(feature = "full-serde")]
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

	/// Get the contract error named `name`, the first if there are multiple.
	pub fn error(&self, name: &str) -> errors::Result<&AbiError> {
		self.errors.get(name).into_iter().flatten().next().ok_or_else(|| Error::InvalidName(name.to_owned()))
	}

	/// Get all contract events named `name`.
	pub fn events_by_name(&self, name: &str) -> errors::Result<&Vec<Event>> {
		self.events.get(name).ok_or_else(|| Error::InvalidName(name.to_owned()))
	}

	/// Get all functions named `name`.
	pub fn functions_by_name(&self, name: &str) -> errors::Result<&Vec<Function>> {
		self.functions.get(name).ok_or_else(|| Error::InvalidName(name.to_owned()))
	}

	/// Get all errors named `name`.
	pub fn errors_by_name(&self, name: &str) -> errors::Result<&Vec<AbiError>> {
		self.errors.get(name).ok_or_else(|| Error::InvalidName(name.to_owned()))
	}

	/// Iterate over all functions of the contract in arbitrary order.
	pub fn functions(&self) -> Functions {
		Functions(self.functions.values().flatten())
	}

	/// Iterate over all events of the contract in arbitrary order.
	pub fn events(&self) -> Events {
		Events(self.events.values().flatten())
	}

	/// Iterate over all errors of the contract in arbitrary order.
	pub fn errors(&self) -> AbiErrors {
		AbiErrors(self.errors.values().flatten())
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

/// Contract errors iterator.
pub struct AbiErrors<'a>(Flatten<Values<'a, String, Vec<AbiError>>>);

impl<'a> Iterator for AbiErrors<'a> {
	type Item = &'a AbiError;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

#[cfg(all(test, feature = "full-serde"))]
#[allow(deprecated)]
mod test {
	use std::{collections::BTreeMap, iter::FromIterator};

	use crate::{tests::assert_ser_de, AbiError, Constructor, Contract, Event, EventParam, Function, Param, ParamType};

	#[test]
	fn empty() {
		let json = "[]";

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: BTreeMap::new(),
				events: BTreeMap::new(),
				errors: BTreeMap::new(),
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
				functions: BTreeMap::new(),
				events: BTreeMap::new(),
				errors: BTreeMap::new(),
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
				functions: BTreeMap::from_iter(vec![
					(
						"foo".to_string(),
						vec![Function {
							name: "foo".to_string(),
							inputs: vec![Param {
								name: "a".to_string(),
								kind: ParamType::Address,
								internal_type: None,
							}],
							outputs: vec![Param {
								name: "res".to_string(),
								kind: ParamType::Address,
								internal_type: None,
							}],
							constant: false,
							state_mutability: Default::default(),
						}]
					),
					(
						"bar".to_string(),
						vec![Function {
							name: "bar".to_string(),
							inputs: vec![],
							outputs: vec![],
							constant: false,
							state_mutability: Default::default(),
						}]
					),
				]),
				events: BTreeMap::new(),
				errors: BTreeMap::new(),
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
				functions: BTreeMap::from_iter(vec![(
					"foo".to_string(),
					vec![
						Function {
							name: "foo".to_string(),
							inputs: vec![Param {
								name: "a".to_string(),
								kind: ParamType::Address,
								internal_type: None,
							}],
							outputs: vec![Param {
								name: "res".to_string(),
								kind: ParamType::Address,
								internal_type: None,
							}],
							constant: false,
							state_mutability: Default::default(),
						},
						Function {
							name: "foo".to_string(),
							inputs: vec![],
							outputs: vec![],
							constant: false,
							state_mutability: Default::default(),
						},
					]
				)]),
				events: BTreeMap::new(),
				errors: BTreeMap::new(),
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
				functions: BTreeMap::new(),
				events: BTreeMap::from_iter(vec![
					(
						"foo".to_string(),
						vec![Event {
							name: "foo".to_string(),
							inputs: vec![EventParam {
								name: "a".to_string(),
								kind: ParamType::Address,
								indexed: false,
							}],
							anonymous: false,
						}]
					),
					(
						"bar".to_string(),
						vec![Event {
							name: "bar".to_string(),
							inputs: vec![EventParam { name: "a".to_string(), kind: ParamType::Address, indexed: true }],
							anonymous: false,
						}]
					),
				]),
				errors: BTreeMap::new(),
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
				functions: BTreeMap::new(),
				events: BTreeMap::from_iter(vec![(
					"foo".to_string(),
					vec![
						Event {
							name: "foo".to_string(),
							inputs: vec![EventParam {
								name: "a".to_string(),
								kind: ParamType::Address,
								indexed: false,
							}],
							anonymous: false,
						},
						Event {
							name: "foo".to_string(),
							inputs: vec![EventParam { name: "a".to_string(), kind: ParamType::Address, indexed: true }],
							anonymous: false,
						},
					]
				)]),
				errors: BTreeMap::new(),
				receive: false,
				fallback: false,
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn errors() {
		let json = r#"
            [
              {
                "type": "error",
                "inputs": [
                  {
                    "name": "available",
                    "type": "uint256"
                  },
                  {
                    "name": "required",
                    "type": "address"
                  }
                ],
                "name": "foo"
              },
              {
                "type": "error",
                "inputs": [
                  {
                    "name": "a",
                    "type": "uint256"
                  },
                  {
                    "name": "b",
                    "type": "address"
                  }
                ],
                "name": "bar"
              }
            ]
		"#;

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: BTreeMap::new(),
				events: BTreeMap::new(),
				errors: BTreeMap::from_iter(vec![
					(
						"foo".to_string(),
						vec![AbiError {
							name: "foo".to_string(),
							inputs: vec![
								Param {
									name: "available".to_string(),
									kind: ParamType::Uint(256),
									internal_type: None,
								},
								Param { name: "required".to_string(), kind: ParamType::Address, internal_type: None }
							],
						}]
					),
					(
						"bar".to_string(),
						vec![AbiError {
							name: "bar".to_string(),
							inputs: vec![
								Param { name: "a".to_string(), kind: ParamType::Uint(256), internal_type: None },
								Param { name: "b".to_string(), kind: ParamType::Address, internal_type: None }
							],
						}]
					),
				]),
				receive: false,
				fallback: false,
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn errors_overload() {
		let json = r#"
			[
			  {
				"type": "error",
				"inputs": [
				  {
					"name": "a",
					"type": "uint256"
				  }
				],
				"name": "foo"
			  },
			  {
				"type": "error",
				"inputs": [
				  {
					"name": "a",
					"type": "uint256"
				  },
				  {
					"name": "b",
					"type": "address"
				  }
				],
				"name": "foo"
			  }
			]
		"#;

		let deserialized: Contract = serde_json::from_str(json).unwrap();

		assert_eq!(
			deserialized,
			Contract {
				constructor: None,
				functions: BTreeMap::new(),
				events: BTreeMap::new(),
				errors: BTreeMap::from_iter(vec![(
					"foo".to_string(),
					vec![
						AbiError {
							name: "foo".to_string(),
							inputs: vec![Param {
								name: "a".to_string(),
								kind: ParamType::Uint(256),
								internal_type: None,
							}],
						},
						AbiError {
							name: "foo".to_string(),
							inputs: vec![
								Param { name: "a".to_string(), kind: ParamType::Uint(256), internal_type: None },
								Param { name: "b".to_string(), kind: ParamType::Address, internal_type: None }
							],
						},
					]
				),]),
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
				functions: BTreeMap::new(),
				events: BTreeMap::new(),
				errors: BTreeMap::new(),
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
				functions: BTreeMap::new(),
				events: BTreeMap::new(),
				errors: BTreeMap::new(),
				receive: false,
				fallback: true,
			}
		);

		assert_ser_de(&deserialized);
	}
}
