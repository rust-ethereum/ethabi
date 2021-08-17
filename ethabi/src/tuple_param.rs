// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tuple param type.

use crate::{param_type::Writer, ParamType};
use serde::{
	de::{Error, MapAccess, Visitor},
	ser::SerializeMap,
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

/// Tuple params specification
#[derive(Debug, Clone, PartialEq)]
pub struct TupleParam {
	/// Param name.
	pub name: Option<String>,

	/// Param type.
	pub kind: ParamType,

	/// Additional Internal type.
	pub internal_type: Option<String>,
}

impl<'a> Deserialize<'a> for TupleParam {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'a>,
	{
		deserializer.deserialize_any(TupleParamVisitor)
	}
}

struct TupleParamVisitor;

impl<'a> Visitor<'a> for TupleParamVisitor {
	type Value = TupleParam;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a valid tuple parameter spec")
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'a>,
	{
		let mut name = None;
		let mut kind = None;
		let mut components = None;
		let mut internal_type = None;

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
						return Err(Error::duplicate_field("type"));
					}
					kind = Some(map.next_value()?);
				}
				"internalType" => {
					if internal_type.is_some() {
						return Err(Error::duplicate_field("internalType"));
					}
					internal_type = Some(map.next_value()?);
				}
				"components" => {
					if components.is_some() {
						return Err(Error::duplicate_field("components"));
					}
					let component: Vec<TupleParam> = map.next_value()?;
					components = Some(component)
				}
				_ => {}
			}
		}

		let mut kind = kind.ok_or_else(|| Error::missing_field("kind"))?;
		crate::param::set_tuple_components(&mut kind, components)?;
		Ok(TupleParam { name, kind, internal_type })
	}
}

impl Serialize for TupleParam {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(None)?;
		if let Some(ref internal_type) = self.internal_type {
			map.serialize_entry("internalType", internal_type)?;
		}
		if let Some(name) = &self.name {
			map.serialize_entry("name", name)?;
		}
		map.serialize_entry("type", &Writer::write_for_abi(&self.kind, false))?;
		if let Some(inner_tuple) = crate::param::inner_tuple(&self.kind) {
			map.serialize_key("components")?;
			map.serialize_value(&crate::param::SerializeableParamVec(inner_tuple))?;
		}
		map.end()
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		tests::{assert_json_eq, assert_ser_de},
		ParamType, TupleParam,
	};

	#[test]
	fn param_simple() {
		let s = r#"{
			"name": "foo",
			"type": "address"
		}"#;

		let deserialized: TupleParam = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			TupleParam { name: Some("foo".to_owned()), kind: ParamType::Address, internal_type: None }
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_internal_type() {
		let s = r#"{
		 	"internalType": "struct Verifier.Proof",
			"name": "foo",
			"type": "address"
		}"#;

		let deserialized: TupleParam = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			TupleParam {
				name: Some("foo".to_owned()),
				kind: ParamType::Address,
				internal_type: Some("struct Verifier.Proof".to_string())
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_unnamed() {
		let s = r#"{
			"type": "address"
		}"#;

		let deserialized: TupleParam = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, TupleParam { name: None, kind: ParamType::Address, internal_type: None });

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_tuple() {
		let s = r#"{
			"type": "tuple",
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

		let deserialized: TupleParam = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			TupleParam {
				name: None,
				kind: ParamType::Tuple(vec![ParamType::Uint(48), ParamType::Tuple(vec![ParamType::Address])]),
				internal_type: None
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_tuple_named() {
		let s = r#"{
			"type": "tuple",
			"components": [
				{
					"name": "amount",
					"type": "uint48"
				},
				{
					"name": "things",
					"type": "tuple",
					"components": [
						{
							"name": "baseTupleParam",
							"type": "address"
						}
					]
				}
			]
		}"#;

		let deserialized: TupleParam = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			TupleParam {
				name: None,
				kind: ParamType::Tuple(vec![ParamType::Uint(48), ParamType::Tuple(vec![ParamType::Address])]),
				internal_type: None
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn param_tuple_array() {
		let s = r#"{
			"type": "tuple[]",
			"components": [
				{
					"type": "uint48"
				},
				{
					"type": "address"
				},
				{
					"type": "address"
				}
			]
		}"#;

		let deserialized: TupleParam = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			TupleParam {
				name: None,
				kind: ParamType::Array(Box::new(ParamType::Tuple(vec![
					ParamType::Uint(48),
					ParamType::Address,
					ParamType::Address
				]))),
				internal_type: None
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_array_of_array_of_tuple() {
		let s = r#"{
			"type": "tuple[][]",
			"components": [
				{
					"type": "uint8"
				},
				{
					"type": "uint16"
				}
			]
		}"#;

		let deserialized: TupleParam = serde_json::from_str(s).unwrap();
		assert_eq!(
			deserialized,
			TupleParam {
				name: None,
				kind: ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Tuple(vec![
					ParamType::Uint(8),
					ParamType::Uint(16),
				]))))),
				internal_type: None
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_tuple_fixed_array() {
		let s = r#"{
			"type": "tuple[2]",
			"components": [
				{
					"type": "uint48"
				},
				{
					"type": "address"
				},
				{
					"type": "address"
				}
			]
		}"#;

		let deserialized: TupleParam = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			TupleParam {
				name: None,
				kind: ParamType::FixedArray(
					Box::new(ParamType::Tuple(vec![ParamType::Uint(48), ParamType::Address, ParamType::Address])),
					2
				),
				internal_type: None
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_tuple_with_nested_tuple_arrays() {
		let s = r#"{
			"type": "tuple",
			"components": [
				{
					"type": "tuple[]",
					"components": [
						{
							"type": "address"
						}
					]
				},
				{
					"type": "tuple[42]",
					"components": [
						{
							"type": "address"
						}
					]
				}
			]
		}"#;

		let deserialized: TupleParam = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			TupleParam {
				name: None,
				kind: ParamType::Tuple(vec![
					ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Address]))),
					ParamType::FixedArray(Box::new(ParamType::Tuple(vec![ParamType::Address])), 42,)
				]),
				internal_type: None
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}
}
