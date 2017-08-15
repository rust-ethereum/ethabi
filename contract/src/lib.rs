extern crate ethabi;

pub struct Log;

pub struct Filter;

pub struct Event;

pub struct Function<'a> {
	pub function: &'a ethabi::Function,
}

impl<'a> Function<'a> {
	pub fn encode(self) -> Vec<u8> {
		unimplemented!();
	}
}

#[macro_export]
macro_rules! use_contract {
	($module: ident, $name: expr, $path: expr) => {
		pub mod $module {
			#[derive(EthabiContract)]
			#[ethabi_contract_options(name = $name, path = $path)]
			struct _Dummy;
		}
	}
}
