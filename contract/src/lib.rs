#[macro_export]
macro_rules! use_contract {
	($module: ident, $name: expr, $path: expr) => {
		#[allow(dead_code)]
		#[allow(missing_docs)]
		#[allow(unused_imports)]
		#[allow(unused_mut)]
		#[allow(unused_variables)]
		pub mod $module {
			#[derive(EthabiContract)]
			#[ethabi_contract_options(name = $name, path = $path)]
			struct _Dummy;
		}
	}
}
