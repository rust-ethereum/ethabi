use std::collections::HashMap;
use spec::{Interface, Operation};
use function::Function;
use constructor::Constructor;
use event::Event;
use error::Error;

/// API building calls to contracts ABI.
#[derive(Clone, Debug)]
pub struct Contract {
    constructor: Option<Constructor>,
    functions: HashMap<String, Function>,
    events: HashMap<String, Event>,
}

impl Contract {
    /// Initializes contract with ABI specification.
    pub fn new(interface: Interface) -> Self {
        let constructor = interface
            .operations()
            .filter_map(Operation::constructor)
            .cloned()
            .map(Constructor::new)
            .next();

        let functions = interface
            .operations()
            .filter_map(Operation::function)
            .cloned()
            .map(|f| (f.name.clone(), Function::new(f)))
            .collect();

        let events = interface
            .operations()
            .filter_map(Operation::event)
            .cloned()
            .map(|e| (e.name.clone(), Event::new(e)))
            .collect();

        Contract {
            constructor,
            events,
            functions,
        }
    }

    /// Creates constructor call builder.
    pub fn constructor(&self) -> Option<Constructor> {
        self.constructor.clone()
    }

    /// Creates function call builder.
    pub fn function(&self, name: &str) -> Result<Function, Error> {
        self.functions.get(name).cloned().ok_or(Error::InvalidName)
    }

    /// Creates event decoder.
    pub fn event(&self, name: &str) -> Result<Event, Error> {
        self.events.get(name).cloned().ok_or(Error::InvalidName)
    }

    /// Iterate over all functions of the contract in arbitrary order.
    pub fn functions<'a>(&'a self) -> Box<Iterator<Item = Function> + 'a> {
        let iter = self.functions.values().cloned();
        Box::new(iter)
    }

    /// Iterate over all events of the contract in arbitrary order.
    pub fn events<'a>(&'a self) -> Box<Iterator<Item = Event> + 'a> {
        let iter = self.events.values().cloned();
        Box::new(iter)
    }
}
