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
	}
}
