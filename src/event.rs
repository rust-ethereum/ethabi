//! Contract event.

use std::collections::HashMap;
use tiny_keccak::keccak256;
use spec::{Event as EventInterface, ParamType, EventParam};
use decoder::Decoder;
use token::Token;
use error::Error;
use signature::long_signature;
use Encoder;

/// Decoded log param.
#[derive(Debug, PartialEq)]
pub struct LogParam {
	/// Decoded log name.
	pub name: String,
	/// Decoded log value.
	pub value: Token,
}

/// Contract event.
#[derive(Clone, Debug)]
pub struct Event {
	/// spec::Event
	interface: EventInterface,
}

impl Event {
	/// Creates new instance of `Event`.
	pub fn new(interface: EventInterface) -> Self {
		Event {
			interface: interface
		}
	}

	/// Event signature
	pub fn signature(&self) -> [u8; 32] {
		long_signature(&self.interface.name, &self.interface.param_types())
	}

	/// Creates topic filter
	pub fn create_topics_filter(&self, topics: Vec<Token>) -> Result<Vec<[u8; 32]>, Error> {
		let topic_params = self.interface.indexed_params(true);
		let equal_len = topics.len() == topic_params.len();
		let equal_types = topics.iter().zip(topic_params.iter()).all(|(topic, param)| topic.type_check(&param.kind));
		if !equal_len || !equal_types {
			return Err(Error::InvalidData);
		}

		let mut result = topics.into_iter()
			.map(|topic| {
				let encoded = Encoder::encode(vec![topic]);
				if encoded.len() == 32 {
					let mut data = [0u8; 32];
					data.copy_from_slice(&encoded);
					data
				} else {
					keccak256(&encoded)
				}
			})
			.collect::<Vec<_>>();

		if !self.interface.anonymous {
			result.insert(0, self.signature());
		}

		Ok(result)
	}

	/// Decodes event indexed params and data.
	pub fn decode_log(&self, topics: Vec<[u8; 32]>, data: Vec<u8>) -> Result<Vec<LogParam>, Error> {
		let topics_len = topics.len();
		// obtains all params info
		let topic_params = self.interface.indexed_params(true);
		let data_params = self.interface.indexed_params(false);
		// then take first topic if event is not anonymous
		let to_skip = if self.interface.anonymous {
			0
		} else {
			// verify
			let event_signature = topics.get(0).ok_or(Error::InvalidData)?;
			if event_signature != &self.signature() {
				return Err(Error::InvalidData);
			}
			1
		};

		let topic_types = topic_params.iter()
			.map(|p| p.kind.clone())
			.collect::<Vec<ParamType>>();

		let flat_topics = topics.into_iter()
			.skip(to_skip)
			.flat_map(|t| t.to_vec())
			.collect::<Vec<u8>>();

		let topic_tokens = try!(Decoder::decode(&topic_types, flat_topics));

		// topic may be only a 32 bytes encoded token
		if topic_tokens.len() != topics_len - to_skip {
			return Err(Error::InvalidData);
		}

		let topics_named_tokens = topic_params.into_iter()
			.map(|p| p.name)
			.zip(topic_tokens.into_iter());

		let data_types = data_params.iter()
			.map(|p| p.kind.clone())
			.collect::<Vec<ParamType>>();

		let data_tokens = try!(Decoder::decode(&data_types, data));

		let data_named_tokens = data_params.into_iter()
			.map(|p| p.name)
			.zip(data_tokens.into_iter());

		let named_tokens = topics_named_tokens
			.chain(data_named_tokens)
			.collect::<HashMap<String, Token>>();

		let decoded_params = self.interface.params_names()
			.into_iter()
			.map(|name| LogParam {
				name: name.clone(),
				value: named_tokens.get(&name).unwrap().clone()
			})
			.collect();

		Ok(decoded_params)
	}

	/// Return the name of the event.
	pub fn name(&self) -> &str {
		&self.interface.name
	}

	/// Return the inputs of the event.
	pub fn inputs(&self) -> &[EventParam] {
		&self.interface.inputs
	}
}

#[cfg(test)]
mod tests {
	use hex::FromHex;
	use spec::{Event as EventInterface, EventParam, ParamType};
	use token::{Token, TokenFromHex};
	use signature::long_signature;
	use super::{Event, LogParam};

	#[test]
	fn test_decoding_event() {
		let i = EventInterface {
			name: "foo".to_owned(),
			inputs: vec![EventParam {
				name: "a".to_owned(),
				kind: ParamType::Int(256),
				indexed: false,
			}, EventParam {
				name: "b".to_owned(),
				kind: ParamType::Int(256),
				indexed: true,
			}, EventParam {
				name: "c".to_owned(),
				kind: ParamType::Address,
				indexed: false,
			}, EventParam {
				name: "d".to_owned(),
				kind: ParamType::Address,
				indexed: true,
			}],
			anonymous: false,
		};

		let event = Event::new(i);

		let result = event.decode_log(
			vec![
				long_signature("foo", &[ParamType::Int(256), ParamType::Int(256), ParamType::Address, ParamType::Address]),
				"0000000000000000000000000000000000000000000000000000000000000002".token_from_hex().unwrap(),
				"0000000000000000000000001111111111111111111111111111111111111111".token_from_hex().unwrap(),
			],
			("".to_owned() +
				"0000000000000000000000000000000000000000000000000000000000000003" +
				"0000000000000000000000002222222222222222222222222222222222222222").from_hex().unwrap()
		).unwrap();

		assert_eq!(result, vec![
			("a".to_owned(), Token::Int("0000000000000000000000000000000000000000000000000000000000000003".token_from_hex().unwrap())),
			("b".to_owned(), Token::Int("0000000000000000000000000000000000000000000000000000000000000002".token_from_hex().unwrap())),
			("c".to_owned(), Token::Address("2222222222222222222222222222222222222222".token_from_hex().unwrap())),
			("d".to_owned(), Token::Address("1111111111111111111111111111111111111111".token_from_hex().unwrap())),
		].into_iter().map(|(name, value)| LogParam { name, value }).collect::<Vec<_>>());
	}
}
