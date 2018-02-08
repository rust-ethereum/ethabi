//! Ethereum ABI encoding decoding library.

#![warn(missing_docs)]

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


// Trait for functions will allow some nice meta programming.
// Trait implemented by the struct returned by .balance_of(123)
pub trait EthabiFunction {
    type Output;

    fn encoded(&self) -> Bytes;
    fn output(&self, Bytes) -> Result<Self::Output>;
}

// A caller extension trait can be implemented for EthabiFunction in a separate crate.
// Lets us use .call() and .transact() on .balance_of(123)
// .balance_of(123).call(|encoded_input| send_to_solaris_and_return_output_bytes())
// aka EthabiFunction.call(closure_get_output_from_bytes)
// aka (rewritten par ce trait): closure_get_output.call(encoded_bytes, output_bytes_to_result)
// [donc closure_get_output [Fn] doit implémenter .call, aka Call<Out> voir plus bas]
// -> C::Result
pub trait DelegateCall<O> {
	fn call<C: Call<O>>(self, caller: C) -> C::Result;
	fn transact<T: Transact>(self, caller: T) -> T::Result;
}
impl<O, EF: EthabiFunction<Output=O>> DelegateCall<O> for EF {
    fn call<C: Call<O>>(self, caller: C) -> C::Result {
        caller.call(self.encoded(), move |bytes| self.output(bytes))
    }

    fn transact<T: Transact>(self, caller: T) -> T::Result {
        caller.transact(self.encoded())
    }
}

// Trait for anything that implements .call(bytes, output_decoder)
// aka trait for any closure_get_output_from_bytes
pub trait Call<Out> {
    // TODO do we actually need any bounds here?
    type Result;

    fn call<F>(self, input: Bytes, output_decoder: F) -> Self::Result
        where F: FnOnce(Bytes) -> Result<Out/*, Error*/>;
}

// A blanket implementations would be nice (that's the current call signature).
impl<Out, F> Call<Out> for F where
    F: FnOnce(Bytes) -> Result<Bytes/*, String*/>
{
    type Result = Result<Out/*, Error*/>;

    fn call<FX>(self, input: Bytes, output_decoder: FX) -> Self::Result
		where FX: FnOnce(Bytes) -> Result<Out/*, String*/>
	{
        (self)(input)
            // .map_err(Error::Message)
            .and_then(output_decoder)
    }
}

// Trait for anything that implements .transact(bytes, output_decoder)
// aka trait for any closure_get_output_from_bytes
pub trait Transact {
    type Result;

    fn transact(self, Bytes) -> Self::Result;
}

// Similar blanket impl for transact (although we don't really have bytes as result, just ())
impl<F> Transact for F where
    F: FnOnce(Bytes) -> Result<()/*, String*/>
{
    type Result = Result<()/*, Error*/>;

    fn transact(self, input: Bytes) -> Self::Result
	{
        (self)(input)
            // .map_err(Error::Message)
    }
}
















