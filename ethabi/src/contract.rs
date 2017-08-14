use std::io;
use std::collections::HashMap;
use std::collections::hash_map::Values;
use spec::Interface;
use function::Function;
use constructor::Constructor;
use event::Event;
use errors::{Error, ErrorKind};

/// API building calls to contracts ABI.
#[derive(Clone, Debug, PartialEq)]
pub struct Contract {
    constructor: Option<Constructor>,
    functions: HashMap<String, Function>,
    events: HashMap<String, Event>,
	fallback: bool,
}

impl From<Interface> for Contract {
	fn from(interface: Interface) -> Self {
        Contract {
            constructor: interface.constructor.map(Into::into),
            events: interface.events.into_iter().map(|(name, event)| (name, event.into())).collect(),
            functions: interface.functions.into_iter().map(|(name, func)| (name, func.into())).collect(),
			fallback: interface.fallback,
        }
	}
}

impl Contract {
	/// Loads contract from json.
	pub fn load<T: io::Read>(reader: T) -> Result<Self, Error> {
		Interface::load(reader).map(Into::into)
	}

    /// Creates constructor call builder.
    pub fn constructor(&self) -> Option<&Constructor> {
        self.constructor.as_ref()
    }

    /// Creates function call builder.
    pub fn function(&self, name: &str) -> Result<&Function, Error> {
        self.functions.get(name).ok_or_else(|| ErrorKind::InvalidName(name.to_owned()).into())
    }

    /// Creates event decoder.
    pub fn event(&self, name: &str) -> Result<&Event, Error> {
        self.events.get(name).ok_or_else(|| ErrorKind::InvalidName(name.to_owned()).into())
    }

    /// Iterate over all functions of the contract in arbitrary order.
    pub fn functions(&self) -> Functions {
        Functions(self.functions.values())
    }

    /// Iterate over all events of the contract in arbitrary order.
    pub fn events(&self) -> Events {
        Events(self.events.values())
    }

	/// Returns true if contract has fallback
	pub fn fallback(&self) -> bool {
		self.fallback
	}
}

/// Contract functions interator.
pub struct Functions<'a>(Values<'a, String, Function>);

impl<'a> Iterator for Functions<'a> {
	type Item = &'a Function;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}

/// Contract events interator.
pub struct Events<'a>(Values<'a, String, Event>);

impl<'a> Iterator for Events<'a> {
	type Item = &'a Event;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next()
	}
}
