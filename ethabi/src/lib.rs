//! Ethereum ABI encoding decoding library.

#![warn(missing_docs)]
#![feature(conservative_impl_trait)]

extern crate rustc_hex as hex;
extern crate serde;
extern crate serde_json;
extern crate tiny_keccak;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

#[doc(hidden)]
pub extern crate futures;
extern crate ethereum_types;

pub mod param_type;
pub mod token;
mod constructor;
mod contract;
mod decoder;
mod encoder;
mod errors;
mod event;
mod event_param;
mod filter;
mod function;
mod log;
mod operation;
mod param;
mod signature;
mod util;

pub use param_type::ParamType;
pub use constructor::Constructor;
pub use contract::{Contract, Functions, Events};
pub use token::Token;
pub use errors::{Error, ErrorKind, Result, ResultExt};
pub use encoder::encode;
pub use decoder::decode;
pub use filter::{Topic, TopicFilter, RawTopicFilter};
pub use function::Function;
pub use param::Param;
pub use log::{Log, RawLog, LogParam, ParseLog};
pub use event::Event;
pub use event_param::EventParam;

/// ABI address.
pub type Address = ethereum_types::Address;

/// ABI fixed bytes.
pub type FixedBytes = Vec<u8>;

/// ABI bytes.
pub type Bytes = Vec<u8>;

/// ABI signed integer.
pub type Int = ethereum_types::U256;

/// ABI unsigned integer.
pub type Uint = ethereum_types::U256;

/// Commonly used FixedBytes of size 32
pub type Hash = ethereum_types::H256;

/// Contract functions from ethabi-derive
pub trait EthabiFunction {
	/// Output types of the contract function
    type Output;

	/// Encodes the input for the contract function
    fn encoded(&self) -> Bytes;

	/// Decodes the given bytes output
    fn output(&self, Bytes) -> Result<Self::Output>;
}

/// Trait that adds `.call(caller)` and `.transact(caller)` to contract functions.
/// The functions get delegated to the caller.
pub trait DelegateCall<O> {
	/// Delegate call to caller
	fn call<C: Call<O>>(self, caller: C) -> C::Result;

	/// Delegate transaction to caller
	fn transact<T: Transact>(self, caller: T) -> T::Result;
}
impl<O, E: EthabiFunction<Output=O> + 'static> DelegateCall<O> for E {
    fn call<C: Call<O>>(self, caller: C) -> C::Result {
        caller.call(self.encoded(), move |bytes| self.output(bytes))
    }

    fn transact<T: Transact>(self, caller: T) -> T::Result {
        caller.transact(self.encoded())
    }
}

/// A caller (for example a closure) that takes input bytes and an output decoder,
/// processes internally an output and returns the decoded output.
pub trait Call<Out>: Sized {
    // TODO do we actually need any bounds here?
	/// Return type of the call
    type Result;

	/// Processes the call given input bytes
    fn call<D: 'static>(self, input: Bytes, output_decoder: D) -> Self::Result
        where D: FnOnce(Bytes) -> Result<Out>;
}

// Blanket implementation for closures
use futures::{Future, IntoFuture};
impl<Out: 'static, F, R: 'static> Call<Out> for F where
	R: IntoFuture<Item=Bytes, Error=Error>,
    F: FnOnce(Bytes) -> R,
{
    type Result = Box<Future<Item=Out, Error=Error>>;

    fn call<D: 'static>(self, input: Bytes, output_decoder: D) -> Self::Result
		where D: FnOnce(Bytes) -> Result<Out>
	{
		Box::new(
			(self)(input).into_future().and_then(output_decoder)
		)
	}

}

/// A caller (for example a closure) that takes input bytes and processes them.
pub trait Transact {
	/// Return type of the transaction
    type Result;

	/// Processes the transaction given input bytes
    fn transact(self, Bytes) -> Self::Result;
}
// Blanket implementation for closures.
// Transactions always return () for now.
impl<F> Transact for F where
    F: FnOnce(Bytes) -> Result<()>
{
    type Result = Result<()>;

    fn transact(self, input: Bytes) -> Self::Result
	{
        (self)(input)
    }
}
















