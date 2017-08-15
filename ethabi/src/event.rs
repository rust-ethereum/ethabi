//! Contract event.

use std::collections::HashMap;
use tiny_keccak::keccak256;
use spec::{Event as EventInterface, ParamType, EventParam};
use decoder::Decoder;
use token::Token;
use errors::{Error, ErrorKind};
use signature::long_signature;
use {Log, Hash, RawLog, LogParam, RawTopicFilter, TopicFilter, Topic};
use Encoder;

/// Contract event.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
	/// spec::Event
	interface: EventInterface,
}

impl From<EventInterface> for Event {
	fn from(interface: EventInterface) -> Self {
		Event {
			interface,
		}
	}
}

impl Event {
	/// Event signature
	pub fn signature(&self) -> Hash {
		long_signature(&self.interface.name, &self.interface.param_types())
	}

	/// Creates topic filter
	pub fn create_filter(&self, raw: RawTopicFilter) -> Result<TopicFilter, Error> {
		fn convert_token(token: Token, kind: &ParamType) -> Result<Hash, Error> {
			if !token.type_check(kind) {
				return Err(ErrorKind::InvalidData.into());
			}
			let encoded = Encoder::encode(vec![token]);
			if encoded.len() == 32 {
				let mut data = [0u8; 32];
				data.copy_from_slice(&encoded);
				Ok(data)
			} else {
				Ok(keccak256(&encoded))
			}
		}

		fn convert_topic(topic: Topic<Token>, kind: Option<&ParamType>) -> Result<Topic<Hash>, Error> {
			match topic {
				Topic::Any => Ok(Topic::Any),
				Topic::OneOf(tokens) => match kind {
					None => Err(ErrorKind::InvalidData.into()),
					Some(kind) => {
						let topics = tokens.into_iter()
							.map(|token| convert_token(token, kind))
							.collect::<Result<Vec<_>, _>>()?;
						Ok(Topic::OneOf(topics))
					}
				},
				Topic::This(token) => match kind {
					None => Err(ErrorKind::InvalidData.into()),
					Some(kind) => Ok(Topic::This(convert_token(token, kind)?)),
				}
			}
		}

		let kinds: Vec<_> = self.interface.indexed_params(true).into_iter().map(|param| param.kind).collect();
		let result = if self.interface.anonymous {
			TopicFilter {
				topic0: convert_topic(raw.topic0, kinds.get(0))?,
				topic1: convert_topic(raw.topic1, kinds.get(1))?,
				topic2: convert_topic(raw.topic2, kinds.get(2))?,
				topic3: Topic::Any,
			}
		} else {
			TopicFilter {
				topic0: Topic::This(self.signature()),
				topic1: convert_topic(raw.topic0, kinds.get(0))?,
				topic2: convert_topic(raw.topic1, kinds.get(1))?,
				topic3: convert_topic(raw.topic2, kinds.get(2))?,
			}
		};

		Ok(result)
	}

	/// Decodes event indexed params and data.
	pub fn parse_log(&self, log: RawLog) -> Result<Log, Error> {
		let topics = log.topics;
		let data = log.data;
		let topics_len = topics.len();
		// obtains all params info
		let topic_params = self.interface.indexed_params(true);
		let data_params = self.interface.indexed_params(false);
		// then take first topic if event is not anonymous
		let to_skip = if self.interface.anonymous {
			0
		} else {
			// verify
			let event_signature = topics.get(0).ok_or(ErrorKind::InvalidData)?;
			if event_signature != &self.signature() {
				return Err(ErrorKind::InvalidData.into());
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
			return Err(ErrorKind::InvalidData.into());
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

		let result = Log {
			params: decoded_params,
		};

		Ok(result)
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
	use log::{RawLog, Log};
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

		let event = Event::from(i);

		let log = RawLog {
			topics: vec![
				long_signature("foo", &[ParamType::Int(256), ParamType::Int(256), ParamType::Address, ParamType::Address]),
				"0000000000000000000000000000000000000000000000000000000000000002".token_from_hex().unwrap(),
				"0000000000000000000000001111111111111111111111111111111111111111".token_from_hex().unwrap(),
			],
			data:
			("".to_owned() +
				"0000000000000000000000000000000000000000000000000000000000000003" +
				"0000000000000000000000002222222222222222222222222222222222222222").from_hex().unwrap()
		};
		let result = event.parse_log(log).unwrap();

		assert_eq!(result, Log { params: vec![
			("a".to_owned(), Token::Int("0000000000000000000000000000000000000000000000000000000000000003".token_from_hex().unwrap())),
			("b".to_owned(), Token::Int("0000000000000000000000000000000000000000000000000000000000000002".token_from_hex().unwrap())),
			("c".to_owned(), Token::Address("2222222222222222222222222222222222222222".token_from_hex().unwrap())),
			("d".to_owned(), Token::Address("1111111111111111111111111111111111111111".token_from_hex().unwrap())),
		].into_iter().map(|(name, value)| LogParam { name, value }).collect::<Vec<_>>()});
	}
}
