// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Event param specification.

#[cfg(feature = "full-serde")]
use core::fmt;

#[cfg(feature = "full-serde")]
use serde::{
	de::{Error, MapAccess, Visitor},
	ser::SerializeMap,
	Deserialize, Deserializer, Serialize, Serializer,
};

#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;
use crate::ParamType;
#[cfg(feature = "full-serde")]
use crate::{param_type::Writer, TupleParam};

/// Event param specification.
#[derive(Debug, Clone, PartialEq)]
pub struct EventParam {
	/// Param name.
	pub name: String,
	/// Param type.
	pub kind: ParamType,
	/// Indexed flag. If true, param is used to build block bloom.
	pub indexed: bool,
}

#[cfg(feature = "full-serde")]
impl<'a> Deserialize<'a> for EventParam {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'a>,
	{
		deserializer.deserialize_any(EventParamVisitor)
	}
}

#[cfg(feature = "full-serde")]
struct EventParamVisitor;

#[cfg(feature = "full-serde")]
impl<'a> Visitor<'a> for EventParamVisitor {
	type Value = EventParam;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a valid event parameter spec")
	}

	fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
	where
		V: MapAccess<'a>,
	{
		let mut name = None;
		let mut kind = None;
		let mut indexed = None;
		let mut components = None;

		while let Some(ref key) = map.next_key::<String>()? {
			match key.as_ref() {
				"name" => {
					if name.is_some() {
						return Err(Error::duplicate_field("name"));
					}
					name = Some(map.next_value()?);
				}
				"type" => {
					if kind.is_some() {
						return Err(Error::duplicate_field("kind"));
					}
					kind = Some(map.next_value()?);
				}
				"components" => {
					if components.is_some() {
						return Err(Error::duplicate_field("components"));
					}
					let component: Vec<TupleParam> = map.next_value()?;
					components = Some(component)
				}
				"indexed" => {
					if indexed.is_some() {
						return Err(Error::duplicate_field("indexed"));
					}
					indexed = Some(map.next_value()?);
				}
				_ => {}
			}
		}
		let name = name.ok_or_else(|| Error::missing_field("name"))?;
		let mut kind = kind.ok_or_else(|| Error::missing_field("kind"))?;
		crate::param::set_tuple_components(&mut kind, components)?;
		let indexed = indexed.unwrap_or(false);
		Ok(EventParam { name, kind, indexed })
	}
}

#[cfg(feature = "full-serde")]
impl Serialize for EventParam {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(None)?;
		map.serialize_entry("name", &self.name)?;
		map.serialize_entry("type", &Writer::write_for_abi(&self.kind, false))?;
		map.serialize_entry("indexed", &self.indexed)?;
		if let Some(inner_tuple) = crate::param::inner_tuple(&self.kind) {
			map.serialize_key("components")?;
			map.serialize_value(&crate::param::SerializeableParamVec(inner_tuple))?;
		}
		map.end()
	}
}

#[cfg(all(test, feature = "full-serde"))]
mod tests {
	use crate::{tests::assert_json_eq, EventParam, ParamType};

	#[test]
	fn event_param_deserialization() {
		let s = r#"{
			"name": "foo",
			"type": "address",
			"indexed": true
		}"#;

		let deserialized: EventParam = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, EventParam { name: "foo".to_owned(), kind: ParamType::Address, indexed: true });

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn event_param_tuple_deserialization() {
		let s = r#"{
			"name": "foo",
			"type": "tuple",
			"indexed": true,
			"components": [
				{
					"type": "uint48"
				},
				{
					"type": "tuple",
					"components": [
						{
							"type": "address"
						}
					]
				}
			]
		}"#;

		let deserialized: EventParam = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			EventParam {
				name: "foo".to_owned(),
				kind: ParamType::Tuple(vec![ParamType::Uint(48), ParamType::Tuple(vec![ParamType::Address])]),
				indexed: true,
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn event_param_tuple_array_deserialization() {
		let s = r#"{
			"components": [
				{ "type": "uint256" },
				{ "type": "address" },
				{
					"components": [
						{ "type": "address" },
						{ "type": "address" }
					],
					"type": "tuple"
				},
				{ "type": "uint256" },
				{
					"components": [
						{
							"components": [
								{ "type": "address" },
								{ "type": "bytes" }
							],
							"type": "tuple[]"
						},
						{
							"components": [
								{ "type": "address" },
								{ "type": "uint256" }
							],
							"type": "tuple[]"
						},
						{ "type": "uint256" }
					],
					"type": "tuple[]"
				},
				{ "type": "uint256" }
			],
			"indexed": false,
			"name": "LogTaskSubmitted",
			"type": "tuple"
        }"#;

		let deserialized: EventParam = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			EventParam {
				name: "LogTaskSubmitted".to_owned(),
				kind: ParamType::Tuple(vec![
					ParamType::Uint(256),
					ParamType::Address,
					ParamType::Tuple(vec![ParamType::Address, ParamType::Address]),
					ParamType::Uint(256),
					ParamType::Array(Box::new(ParamType::Tuple(vec![
						ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Address, ParamType::Bytes,]))),
						ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Address, ParamType::Uint(256)]))),
						ParamType::Uint(256),
					]))),
					ParamType::Uint(256),
				]),
				indexed: false,
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}
}
