// Copyright 2015-2019 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_export]
macro_rules! use_contract {
	($module: ident, $path: expr) => {
		#[allow(dead_code)]
		#[allow(missing_docs)]
		#[allow(unused_imports)]
		#[allow(unused_mut)]
		#[allow(unused_variables)]
		pub mod $module {
			#[derive(ethabi_derive::EthabiContract)]
			#[ethabi_contract_options(path = $path)]
			struct _Dummy;
		}
	};
}
