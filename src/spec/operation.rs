//! Operation type.

use serde::{Deserialize, Deserializer};
use serde::de::{Error as SerdeError};
use serde_json::Value;
use serde_json::value::from_value;
use super::{Function, Event, Constructor};

/// Operation type.
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
	/// Contract constructor.
	Constructor(Constructor),
	/// Contract function.
	Function(Function),
	/// Contract event.
	Event(Event),
	/// Fallback, ignored.
	Fallback,
}

impl<'a> Deserialize<'a> for Operation {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
		let v: Value = try!(Deserialize::deserialize(deserializer));
		let cloned = v.clone();
		let map = try!(cloned.as_object().ok_or_else(|| SerdeError::custom("Invalid operation")));
		let s = try!(map.get("type").and_then(Value::as_str).ok_or_else(|| SerdeError::custom("Invalid operation type")));

		// This is a workaround to support non-spec compliant function and event names,
		// see: https://github.com/paritytech/parity/issues/4122
		fn sanitize_name(name: &mut String) {
			if let Some(i) = name.find('(') {
				name.truncate(i);
			}
		}

		let result = match s {
			"constructor" => from_value(v).map(Operation::Constructor),
			"function" => from_value(v).map(|mut f: Function| {
				sanitize_name(&mut f.name);
				Operation::Function(f)
			}),
			"event" => from_value(v).map(|mut e: Event| {
				sanitize_name(&mut e.name);
				Operation::Event(e)
			}),
			"fallback" => Ok(Operation::Fallback),
			_ => Err(SerdeError::custom("Invalid operation type.")),
		};
		result.map_err(|e| D::Error::custom(e.to_string()))
	}
}

impl Operation {
	/// Return some if this operation is a `Constructor`.
	pub fn constructor(&self) -> Option<&Constructor> {
		match *self {
			Operation::Constructor(ref f) => Some(f),
			_ => None
		}
	}

	/// Return some if this operation is a `Function`.
	pub fn function(&self) -> Option<&Function> {
		match *self {
			Operation::Function(ref f) => Some(f),
			_ => None
		}
	}

	/// Return some if this operation is an `Event`.
	pub fn event(&self) -> Option<&Event> {
		match *self {
			Operation::Event(ref e) => Some(e),
			_ => None
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::Operation;
	use spec::{ParamType, Function, Param};

	#[test]
	fn deserialize_operation() {
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

		assert_eq!(deserialized, Operation::Function(Function {
			name: "foo".to_owned(),
			inputs: vec![
				Param {
					name: "a".to_owned(),
					kind: ParamType::Address,
				}
			],
			outputs: vec![]
		}));
	}

	#[test]
	fn deserialize_sanitize_function_name() {
		fn test_sanitize_function_name(name: &str, expected: &str) {
			let s = format!(r#"{{
				"type":"function",
				"inputs": [{{
					"name":"a",
					"type":"address"
				}}],
				"name":"{}",
				"outputs": []
			}}"#, name);

			let deserialized: Operation = serde_json::from_str(&s).unwrap();
			let function = deserialized.function().unwrap();

			assert_eq!(function.name, expected);
		}

		test_sanitize_function_name("foo", "foo");
		test_sanitize_function_name("foo()", "foo");
		test_sanitize_function_name("()", "");
		test_sanitize_function_name("", "");
	}

	#[test]
	fn deserialize_sanitize_event_name() {
		fn test_sanitize_event_name(name: &str, expected: &str) {
			let s = format!(r#"{{
				"type":"event",
					"inputs": [{{
						"name":"a",
						"type":"address",
						"indexed":true
					}}],
					"name":"{}",
					"outputs": [],
					"anonymous": false
			}}"#, name);

			let deserialized: Operation = serde_json::from_str(&s).unwrap();
			let event = deserialized.event().unwrap();

			assert_eq!(event.name, expected);
		}

		test_sanitize_event_name("foo", "foo");
		test_sanitize_event_name("foo()", "foo");
		test_sanitize_event_name("()", "");
		test_sanitize_event_name("", "");
	}
}
