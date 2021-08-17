// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Function param.

use serde::{
	de::{Error, MapAccess, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

use crate::{param_type::Writer, ParamType, TupleParam};
use serde::ser::{SerializeMap, SerializeSeq};

/// Function param.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
	/// Param name.
	pub name: String,
	/// Param type.
	pub kind: ParamType,
	/// Additional Internal type.
	pub internal_type: Option<String>,
}

impl<'a> Deserialize<'a> for Param {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'a>,
	{
		deserializer.deserialize_any(ParamVisitor)
	}
}

struct ParamVisitor;

impl<'a> Visitor<'a> for ParamVisitor {
	type Value = Param;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a valid event parameter spec")
	}

	fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
	where
		V: MapAccess<'a>,
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
						return Err(Error::duplicate_field("kind"));
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
		let name = name.ok_or_else(|| Error::missing_field("name"))?;
		let mut kind = kind.ok_or_else(|| Error::missing_field("kind"))?;
		set_tuple_components::<V::Error>(&mut kind, components)?;
		Ok(Param { name, kind, internal_type })
	}
}

impl Serialize for Param {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(None)?;
		if let Some(ref internal_type) = self.internal_type {
			map.serialize_entry("internalType", internal_type)?;
		}
		map.serialize_entry("name", &self.name)?;
		map.serialize_entry("type", &Writer::write_for_abi(&self.kind, false))?;
		if let Some(inner_tuple) = inner_tuple(&self.kind) {
			map.serialize_key("components")?;
			map.serialize_value(&SerializeableParamVec(inner_tuple))?;
		}
		map.end()
	}
}

pub(crate) fn inner_tuple_mut(mut param: &mut ParamType) -> Option<&mut Vec<ParamType>> {
	loop {
		match param {
			ParamType::Array(inner) => param = inner.as_mut(),
			ParamType::FixedArray(inner, _) => param = inner.as_mut(),
			ParamType::Tuple(inner) => return Some(inner),
			_ => return None,
		}
	}
}

pub(crate) fn inner_tuple(mut param: &ParamType) -> Option<&Vec<ParamType>> {
	loop {
		match param {
			ParamType::Array(inner) => param = inner.as_ref(),
			ParamType::FixedArray(inner, _) => param = inner.as_ref(),
			ParamType::Tuple(inner) => return Some(inner),
			_ => return None,
		}
	}
}

pub(crate) fn set_tuple_components<Error: serde::de::Error>(
	kind: &mut ParamType,
	components: Option<Vec<TupleParam>>,
) -> Result<(), Error> {
	if let Some(inner_tuple_mut) = inner_tuple_mut(kind) {
		let tuple_params = components.ok_or_else(|| Error::missing_field("components"))?;
		inner_tuple_mut.extend(tuple_params.into_iter().map(|param| param.kind))
	}
	Ok(())
}

pub(crate) struct SerializeableParamVec<'a>(pub(crate) &'a [ParamType]);

impl Serialize for SerializeableParamVec<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut seq = serializer.serialize_seq(None)?;
		for param in self.0 {
			seq.serialize_element(&SerializeableParam(param))?;
		}
		seq.end()
	}
}

pub(crate) struct SerializeableParam<'a>(pub(crate) &'a ParamType);

impl Serialize for SerializeableParam<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(None)?;
		map.serialize_entry("type", &Writer::write_for_abi(self.0, false))?;
		if let Some(inner_tuple) = inner_tuple(self.0) {
			map.serialize_key("components")?;
			map.serialize_value(&SerializeableParamVec(inner_tuple))?;
		}
		map.end()
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		tests::{assert_json_eq, assert_ser_de},
		Param, ParamType,
	};

	#[test]
	fn param_simple() {
		let s = r#"{
			"name": "foo",
			"type": "address"
		}"#;

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Param { name: "foo".to_owned(), kind: ParamType::Address, internal_type: None });

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_simple_internal_type() {
		let s = r#"{
			"name": "foo",
			"type": "address",
			"internalType": "struct Verifier.Proof"
		}"#;

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			Param {
				name: "foo".to_owned(),
				kind: ParamType::Address,
				internal_type: Some("struct Verifier.Proof".to_string())
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_tuple() {
		let s = r#"{
			"name": "foo",
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

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			Param {
				name: "foo".to_owned(),
				kind: ParamType::Tuple(vec![ParamType::Uint(48), ParamType::Tuple(vec![ParamType::Address])]),
				internal_type: None
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_tuple_internal_type() {
		let s = r#"{
			"name": "foo",
			"type": "tuple",
			"internalType": "struct Pairing.G1Point[]",
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

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			Param {
				name: "foo".to_owned(),
				kind: ParamType::Tuple(vec![ParamType::Uint(48), ParamType::Tuple(vec![ParamType::Address])]),
				internal_type: Some("struct Pairing.G1Point[]".to_string())
			}
		);

		assert_json_eq(s, serde_json::to_string(&deserialized).unwrap().as_str());
	}

	#[test]
	fn param_tuple_named() {
		let s = r#"{
			"name": "foo",
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

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			Param {
				name: "foo".to_owned(),
				kind: ParamType::Tuple(vec![ParamType::Uint(48), ParamType::Tuple(vec![ParamType::Address])]),
				internal_type: None
			}
		);

		assert_ser_de(&deserialized);
	}

	#[test]
	fn param_tuple_array() {
		let s = r#"{
			"name": "foo",
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

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			Param {
				name: "foo".to_owned(),
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
			"name": "foo",
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

		let deserialized: Param = serde_json::from_str(s).unwrap();
		assert_eq!(
			deserialized,
			Param {
				name: "foo".to_owned(),
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
			"name": "foo",
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

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			Param {
				name: "foo".to_owned(),
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
			"name": "foo",
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

		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(
			deserialized,
			Param {
				name: "foo".to_owned(),
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
