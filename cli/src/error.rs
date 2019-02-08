#![allow(unknown_lints)]

use std::io;
use {ethabi, docopt, hex};
use ethabi::Hash;

error_chain! {
	links {
		Ethabi(ethabi::Error, ethabi::ErrorKind);
	}

	foreign_links {
		Io(io::Error);
		Docopt(docopt::Error);
		Hex(hex::FromHexError);
	}

	errors {
		InvalidSignature(signature: Hash) {
			description("Invalid signature"),
			display("Invalid signature `{}`", signature),
		}

		AmbiguousEventName(name: String) {
			description("More than one event found for name, try providing the full signature"),
			display("Ambiguous event name `{}`", name),
		}
	}
}
