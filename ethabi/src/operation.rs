// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Operation type.

use crate::{Constructor, Event, Function};
use serde::{Deserialize, Serialize};

/// Operation type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Operation {
	/// Contract constructor.
	#[serde(rename = "constructor")]
	Constructor(Constructor),
	/// Contract function.
	#[serde(rename = "function")]
	Function(Function),
	/// Contract event.
	#[serde(rename = "event")]
	Event(Event),
	/// Fallback function.
	#[serde(rename = "fallback")]
	Fallback,
	/// Receive function.
	#[serde(rename = "receive")]
	Receive,
}

#[cfg(test)]
mod tests {
	use super::Operation;
	use crate::{tests::assert_ser_de, Event, EventParam, Function, Param, ParamType, StateMutability};

	#[test]
	fn operation() {
		let s = r#"{
			"type":"function",
			"inputs": [{
				"name":"a",
				"type":"address"
			}],
			"name":"foo",
			"outputs": []
		}"#;

		let deserialized: Operation = serde_json::from_str(s).unwrap();

		#[allow(deprecated)]
		let function = Function {
			name: "foo".to_owned(),
			inputs: vec![Param { name: "a".to_owned(), kind: ParamType::Address, internal_type: None }],
			outputs: vec![],
			constant: false,
			state_mutability: StateMutability::NonPayable,
		};
		assert_eq!(deserialized, Operation::Function(function));

		assert_ser_de(&deserialized);
	}

	#[test]
	fn event_operation_with_tuple_array_input() {
		let s = r#"{
			"type":"event",
			"inputs": [
				{
					"name":"a",
					"type":"address",
					"indexed":true
				},
				{
				  "components": [
					{
					  "internalType": "address",
					  "name": "to",
					  "type": "address"
					},
					{
					  "internalType": "uint256",
					  "name": "value",
					  "type": "uint256"
					},
					{
					  "internalType": "bytes",
					  "name": "data",
					  "type": "bytes"
					}
				  ],
				  "indexed": false,
				  "internalType": "struct Action[]",
				  "name": "b",
				  "type": "tuple[]"
				}
			],
			"name":"E",
			"outputs": [],
			"anonymous": false
		}"#;

		let deserialized: Operation = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			Operation::Event(Event {
				name: "E".to_owned(),
				inputs: vec![
					EventParam { name: "a".to_owned(), kind: ParamType::Address, indexed: true },
					EventParam {
						name: "b".to_owned(),
						kind: ParamType::Array(Box::new(ParamType::Tuple(vec![
							ParamType::Address,
							ParamType::Uint(256),
							ParamType::Bytes
						]))),
						indexed: false
					},
				],
				anonymous: false,
			})
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn sanitize_function_name() {
		fn test_sanitize_function_name(name: &str, expected: &str) {
			let s = format!(
				r#"{{
					"type":"function",
					"inputs": [{{
						"name":"a",
						"type":"address"
					}}],
					"name":"{}",
					"outputs": []
				}}"#,
				name
			);

			let deserialized: Operation = serde_json::from_str(&s).unwrap();
			let function = match &deserialized {
				Operation::Function(f) => f,
				_ => panic!("expected funciton"),
			};

			assert_eq!(function.name, expected);

			assert_ser_de(&deserialized);
		}

		test_sanitize_function_name("foo", "foo");
		test_sanitize_function_name("foo()", "foo");
		test_sanitize_function_name("()", "");
		test_sanitize_function_name("", "");
	}

	#[test]
	fn sanitize_event_name() {
		fn test_sanitize_event_name(name: &str, expected: &str) {
			let s = format!(
				r#"{{
					"type":"event",
						"inputs": [{{
							"name":"a",
							"type":"address",
							"indexed":true
						}}],
						"name":"{}",
						"outputs": [],
						"anonymous": false
				}}"#,
				name
			);

			let deserialized: Operation = serde_json::from_str(&s).unwrap();
			let event = match deserialized {
				Operation::Event(e) => e,
				_ => panic!("expected event!"),
			};

			assert_eq!(event.name, expected);

			assert_ser_de(&Operation::Event(event));
		}

		test_sanitize_event_name("foo", "foo");
		test_sanitize_event_name("foo()", "foo");
		test_sanitize_event_name("()", "");
		test_sanitize_event_name("", "");
	}
}
