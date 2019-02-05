#![allow(unknown_lints)]
#![allow(missing_docs)]

use std::{num, string};
use {serde_json, hex};
use Hash;

error_chain! {
	foreign_links {
		SerdeJson(serde_json::Error);
		ParseInt(num::ParseIntError);
		Utf8(string::FromUtf8Error);
		Hex(hex::FromHexError);
	}

	errors {
		InvalidName(name: String) {
			description("Invalid name"),
			display("Invalid name `{}`", name),
		}

		InvalidData {
			description("Invalid data"),
			display("Invalid data"),
		}

		InvalidSignature(signature: Hash) {
			description("Invalid signature"),
			display("Invalid signature `{}`", signature),
		}
	}
}
