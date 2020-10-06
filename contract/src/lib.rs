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

	(@inner $module:ident, $path: expr, ($($signature:expr => $alias:expr),*)) => {
        #[allow(dead_code)]
		#[allow(missing_docs)]
		#[allow(unused_imports)]
		#[allow(unused_mut)]
		#[allow(unused_variables)]
		pub mod $module {
			#[derive(ethabi_derive::EthabiContract)]
			#[ethabi_contract_options(path = $path)]
			$(#[ethabi_function_options(signature = $signature, alias = $alias)])*
			struct _Dummy;
		}
    };

    // match for the extra comma in the last parameter:
    // use_contract!{erc721, "../res/ERC721.abi",
    //     "safeTransferFrom(address,address,uint256,bytes)" => "safe_transfer_with_data",
    // }
    ($module:ident, $path: expr, $($signature:expr => $alias:expr,)*) => {
        use_contract!(@inner $module, $path, ($($signature => $alias),*));
    };

    // match for:
    // use_contract!{erc721, "../res/ERC721.abi",
    //     "safeTransferFrom(address,address,uint256,bytes)" => "safe_transfer_with_data"
    // }
    ($module:ident, $path: expr, $($signature:expr => $alias:expr),*) => {
        use_contract!(@inner $module, $path, ($($signature => $alias),*));
    };
}
