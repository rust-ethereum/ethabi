// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Function and event param types.

#[cfg(feature = "serde")]
mod deserialize;

mod param_type;
pub use param_type::ParamType;

#[cfg(feature = "serde")]
mod reader;
#[cfg(feature = "serde")]
pub use reader::Reader;

mod writer;
pub use writer::Writer;
