pub extern crate futures;

#[macro_export]
macro_rules! use_contract {
	($module: ident, $name: expr, include_str!($abi_path: expr)) => {
		#[allow(dead_code)]
		pub mod $module {
			#[derive(EthabiContract)]
			#[ethabi_contract_options(name = $name, abi_path = $abi_path)]
			struct _Dummy;
		}
	};
	($module: ident, $name: expr, $abi_inline: expr) => {
		#[allow(dead_code)]
		pub mod $module {
			#[derive(EthabiContract)]
			#[ethabi_contract_options(name = $name, abi_inline = $abi_inline)]
			struct _Dummy;
		}
	};
}
