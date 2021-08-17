use anyhow::anyhow;
use ethabi::{
	decode, encode,
	param_type::{ParamType, Reader},
	token::{LenientTokenizer, StrictTokenizer, Token, Tokenizer},
	Contract, Event, Function, Hash,
};
use itertools::Itertools;
use sha3::{Digest, Keccak256};
use std::fs::File;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
/// Ethereum ABI coder.
enum Opt {
	/// Encode ABI call.
	Encode(Encode),
	/// Decode ABI call result.
	Decode(Decode),
}

#[derive(StructOpt, Debug)]
enum Encode {
	/// Load function from JSON ABI file.
	Function {
		abi_path: String,
		function_name_or_signature: String,
		#[structopt(short, number_of_values = 1)]
		params: Vec<String>,
		/// Allow short representation of input params.
		#[structopt(short, long)]
		lenient: bool,
	},
	/// Specify types of input params inline.
	Params {
		/// Pairs of types directly followed by params in the form:
		///
		/// -v <type1> <param1> -v <type2> <param2> ...
		#[structopt(short = "v", name = "type-or-param", number_of_values = 2, allow_hyphen_values = true)]
		params: Vec<String>,
		/// Allow short representation of input params (numbers are in decimal form).
		#[structopt(short, long)]
		lenient: bool,
	},
}

#[derive(StructOpt, Debug)]
enum Decode {
	/// Load function from JSON ABI file.
	Function { abi_path: String, function_name_or_signature: String, data: String },
	/// Specify types of input params inline.
	Params {
		#[structopt(short, name = "type", number_of_values = 1)]
		types: Vec<String>,
		data: String,
	},
	/// Decode event log.
	Log {
		abi_path: String,
		event_name_or_signature: String,
		#[structopt(short = "l", name = "topic", number_of_values = 1)]
		topics: Vec<String>,
		data: String,
	},
}

fn main() -> anyhow::Result<()> {
	println!("{}", execute(std::env::args())?);

	Ok(())
}

fn execute<I>(args: I) -> anyhow::Result<String>
where
	I: IntoIterator,
	I::Item: Into<std::ffi::OsString> + Clone,
{
	let opt = Opt::from_iter(args);

	match opt {
		Opt::Encode(Encode::Function { abi_path, function_name_or_signature, params, lenient }) => {
			encode_input(&abi_path, &function_name_or_signature, &params, lenient)
		}
		Opt::Encode(Encode::Params { params, lenient }) => encode_params(&params, lenient),
		Opt::Decode(Decode::Function { abi_path, function_name_or_signature, data }) => {
			decode_call_output(&abi_path, &function_name_or_signature, &data)
		}
		Opt::Decode(Decode::Params { types, data }) => decode_params(&types, &data),
		Opt::Decode(Decode::Log { abi_path, event_name_or_signature, topics, data }) => {
			decode_log(&abi_path, &event_name_or_signature, &topics, &data)
		}
	}
}

fn load_function(path: &str, name_or_signature: &str) -> anyhow::Result<Function> {
	let file = File::open(path)?;
	let contract = Contract::load(file)?;
	let params_start = name_or_signature.find('(');

	match params_start {
		// It's a signature
		Some(params_start) => {
			let name = &name_or_signature[..params_start];

			contract
				.functions_by_name(name)?
				.iter()
				.find(|f| f.signature() == name_or_signature)
				.cloned()
				.ok_or_else(|| anyhow!("invalid function signature `{}`", name_or_signature))
		}

		// It's a name
		None => {
			let functions = contract.functions_by_name(name_or_signature)?;
			match functions.len() {
				0 => unreachable!(),
				1 => Ok(functions[0].clone()),
				_ => Err(anyhow!(
					"More than one function found for name `{}`, try providing the full signature",
					name_or_signature
				)),
			}
		}
	}
}

fn load_event(path: &str, name_or_signature: &str) -> anyhow::Result<Event> {
	let file = File::open(path)?;
	let contract = Contract::load(file)?;
	let params_start = name_or_signature.find('(');

	match params_start {
		// It's a signature.
		Some(params_start) => {
			let name = &name_or_signature[..params_start];
			let signature = hash_signature(name_or_signature);
			contract
				.events_by_name(name)?
				.iter()
				.find(|event| event.signature() == signature)
				.cloned()
				.ok_or_else(|| anyhow!("Invalid signature `{}`", signature))
		}

		// It's a name.
		None => {
			let events = contract.events_by_name(name_or_signature)?;
			match events.len() {
				0 => unreachable!(),
				1 => Ok(events[0].clone()),
				_ => Err(anyhow!(
					"More than one function found for name `{}`, try providing the full signature",
					name_or_signature
				)),
			}
		}
	}
}

fn parse_tokens(params: &[(ParamType, &str)], lenient: bool) -> anyhow::Result<Vec<Token>> {
	params
		.iter()
		.map(|&(ref param, value)| match lenient {
			true => LenientTokenizer::tokenize(param, value),
			false => StrictTokenizer::tokenize(param, value),
		})
		.collect::<Result<_, _>>()
		.map_err(From::from)
}

fn encode_input(path: &str, name_or_signature: &str, values: &[String], lenient: bool) -> anyhow::Result<String> {
	let function = load_function(path, name_or_signature)?;

	let params: Vec<_> =
		function.inputs.iter().map(|param| param.kind.clone()).zip(values.iter().map(|v| v as &str)).collect();

	let tokens = parse_tokens(&params, lenient)?;
	let result = function.encode_input(&tokens)?;

	Ok(hex::encode(&result))
}

fn encode_params(params: &[String], lenient: bool) -> anyhow::Result<String> {
	assert_eq!(params.len() % 2, 0);

	let params = params
		.iter()
		.tuples::<(_, _)>()
		.map(|(x, y)| Reader::read(x).map(|z| (z, y.as_str())))
		.collect::<Result<Vec<_>, _>>()?;

	let tokens = parse_tokens(params.as_slice(), lenient)?;
	let result = encode(&tokens);

	Ok(hex::encode(&result))
}

fn decode_call_output(path: &str, name_or_signature: &str, data: &str) -> anyhow::Result<String> {
	let function = load_function(path, name_or_signature)?;
	let data: Vec<u8> = hex::decode(&data)?;
	let tokens = function.decode_output(&data)?;
	let types = function.outputs;

	assert_eq!(types.len(), tokens.len());

	let result = types
		.iter()
		.zip(tokens.iter())
		.map(|(ty, to)| format!("{} {}", ty.kind, to))
		.collect::<Vec<String>>()
		.join("\n");

	Ok(result)
}

fn decode_params(types: &[String], data: &str) -> anyhow::Result<String> {
	let types: Vec<ParamType> = types.iter().map(|s| Reader::read(s)).collect::<Result<_, _>>()?;

	let data: Vec<u8> = hex::decode(&data)?;

	let tokens = decode(&types, &data)?;

	assert_eq!(types.len(), tokens.len());

	let result =
		types.iter().zip(tokens.iter()).map(|(ty, to)| format!("{} {}", ty, to)).collect::<Vec<String>>().join("\n");

	Ok(result)
}

fn decode_log(path: &str, name_or_signature: &str, topics: &[String], data: &str) -> anyhow::Result<String> {
	let event = load_event(path, name_or_signature)?;
	let topics: Vec<Hash> = topics.iter().map(|t| t.parse()).collect::<Result<_, _>>()?;
	let data = hex::decode(data)?;
	let decoded = event.parse_log((topics, data).into())?;

	let result = decoded
		.params
		.into_iter()
		.map(|log_param| format!("{} {}", log_param.name, log_param.value))
		.collect::<Vec<String>>()
		.join("\n");

	Ok(result)
}

fn hash_signature(sig: &str) -> Hash {
	Hash::from_slice(Keccak256::digest(sig.replace(" ", "").as_bytes()).as_slice())
}

#[cfg(test)]
mod tests {
	use super::execute;

	#[test]
	fn simple_encode() {
		let command = "ethabi encode params -v bool 1".split(' ');
		let expected = "0000000000000000000000000000000000000000000000000000000000000001";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn int_encode() {
		let command = "ethabi encode params -v int256 -2 --lenient".split(' ');
		let expected = "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe";
		assert_eq!(execute(command).unwrap(), expected);

		let command = "ethabi encode params -v int256 -0 --lenient".split(' ');
		let expected = "0000000000000000000000000000000000000000000000000000000000000000";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn uint_encode_must_be_positive() {
		let command = "ethabi encode params -v uint256 -2 --lenient".split(' ');
		assert!(execute(command).is_err());
	}

	#[test]
	fn uint_encode_requires_decimal_inputs() {
		let command = "ethabi encode params -v uint256 123abc --lenient".split(' ');
		let result = execute(command);
		assert!(result.is_err());
		let err = result.unwrap_err();
		assert_eq!(err.to_string(), "Uint parse error: InvalidCharacter");
	}

	#[test]
	fn uint_encode_big_numbers() {
		let command =
			"ethabi encode params -v uint256 100000000000000000000000000000000022222222222222221111111111111 --lenient"
				.split(' ');
		let expected = "0000000000003e3aeb4ae1383562f4b82261d96a3f7a5f62ca19599c1ad6d1c7";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn int_encode_large_negative_numbers() {
		// i256::min_value() is ok
		let command = "ethabi encode params -v int256 -57896044618658097711785492504343953926634992332820282019728792003956564819968 --lenient".split(' ');
		let expected = "8000000000000000000000000000000000000000000000000000000000000000";
		assert_eq!(execute(command).unwrap(), expected);

		// i256::min_value() - 1 is too much
		let command = "ethabi encode params -v int256 -57896044618658097711785492504343953926634992332820282019728792003956564819969 --lenient".split(' ');
		assert_eq!(execute(command).unwrap_err().to_string(), "int256 parse error: Underflow");
	}

	#[test]
	fn int_encode_large_positive_numbers() {
		// Overflow
		let command = "ethabi encode params -v int256 100000000000000000000000000000000022222222222222221111111111111333333333344556 --lenient".split(' ');
		assert_eq!(execute(command).unwrap_err().to_string(), "int256 parse error: Overflow");

		// i256::max_value() is ok
		let command = "ethabi encode params -v int256 57896044618658097711785492504343953926634992332820282019728792003956564819967 --lenient".split(' ');
		let expected = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
		assert_eq!(execute(command).unwrap(), expected);

		// i256::max_value() + 1 is too much
		let command = "ethabi encode params -v int256 57896044618658097711785492504343953926634992332820282019728792003956564819968 --lenient".split(' ');
		assert_eq!(execute(command).unwrap_err().to_string(), "int256 parse error: Overflow");
	}

	#[test]
	fn multi_encode() {
		let command = "ethabi encode params -v bool 1 -v string gavofyork -v bool 0".split(' ');
		let expected = "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000096761766f66796f726b0000000000000000000000000000000000000000000000";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn array_encode() {
		let command = "ethabi encode params -v bool[] [1,0,false]".split(' ');
		let expected = "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn function_encode_by_name() {
		let command = "ethabi encode function ../res/test.abi foo -p 1".split(' ');
		let expected = "455575780000000000000000000000000000000000000000000000000000000000000001";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn function_encode_by_signature() {
		let command = "ethabi encode function ../res/test.abi foo(bool) -p 1".split(' ');
		let expected = "455575780000000000000000000000000000000000000000000000000000000000000001";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn nonexistent_function() {
		// This should fail because there is no function called 'nope' in the ABI
		let command = "ethabi encode function ../res/test.abi nope -p 1".split(' ');
		assert!(execute(command).is_err());
	}

	#[test]
	fn overloaded_function_encode_by_name() {
		// This should fail because there are two definitions of `bar in the ABI
		let command = "ethabi encode function ../res/test.abi bar -p 1".split(' ');
		assert!(execute(command).is_err());
	}

	#[test]
	fn overloaded_function_encode_by_first_signature() {
		let command = "ethabi encode function ../res/test.abi bar(bool) -p 1".split(' ');
		let expected = "6fae94120000000000000000000000000000000000000000000000000000000000000001";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn overloaded_function_encode_by_second_signature() {
		let command = "ethabi encode function ../res/test.abi bar(string):(uint256) -p 1".split(' ');
		let expected = "d473a8ed0000000000000000000000000000000000000000000000000000000000000020\
		                000000000000000000000000000000000000000000000000000000000000000131000000\
		                00000000000000000000000000000000000000000000000000000000";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn simple_decode() {
		let command =
			"ethabi decode params -t bool 0000000000000000000000000000000000000000000000000000000000000001".split(' ');
		let expected = "bool true";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn int_decode() {
		let command = "ethabi decode params -t int256 fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"
			.split(' ');
		let expected = "int256 fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn multi_decode() {
		let command = "ethabi decode params -t bool -t string -t bool 00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000096761766f66796f726b0000000000000000000000000000000000000000000000".split(' ');
		let expected = "bool true
string gavofyork
bool false";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn array_decode() {
		let command = "ethabi decode params -t bool[] 00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".split(' ');
		let expected = "bool[] [true,false,false]";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn abi_decode() {
		let command = "ethabi decode function ../res/foo.abi bar 0000000000000000000000000000000000000000000000000000000000000001".split(' ');
		let expected = "bool true";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn log_decode() {
		let command = "ethabi decode log ../res/event.abi Event -l 0000000000000000000000000000000000000000000000000000000000000001 0000000000000000000000004444444444444444444444444444444444444444".split(' ');
		let expected = "a true
b 4444444444444444444444444444444444444444";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn log_decode_signature() {
		let command = "ethabi decode log ../res/event.abi Event(bool,address) -l 0000000000000000000000000000000000000000000000000000000000000001 0000000000000000000000004444444444444444444444444444444444444444".split(' ');
		let expected = "a true
b 4444444444444444444444444444444444444444";
		assert_eq!(execute(command).unwrap(), expected);
	}

	#[test]
	fn nonexistent_event() {
		// This should return an error because no event 'Nope(bool,address)' exists
		let command = "ethabi decode log ../res/event.abi Nope(bool,address) -l 0000000000000000000000000000000000000000000000000000000000000000 0000000000000000000000004444444444444444444444444444444444444444".split(' ');
		assert!(execute(command).is_err());
	}
}
