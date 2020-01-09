use std::{fmt, io};

/// Ethabi CLI errors
#[derive(Debug)]
pub enum Error {
    /// Invalid signature.
    InvalidSignature(ethabi::Hash),
    /// Ambiguous event name.
    AmbiguousEventName(String),
    /// Invalid function signature.
    InvalidFunctionSignature(String),
    /// Ambiguous function name.
    AmbiguousFunctionName(String),
    /// ABI error.
    Ethabi(ethabi::Error),
    /// IO error.
    Io(io::Error),
    /// Hex parsing error.
    Hex(rustc_hex::FromHexError),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;
        match self {
            Ethabi(e) => Some(e),
            Io(e) => Some(e),
            Hex(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Error::*;
        match *self {
            InvalidSignature(ref signature) => write!(f, "Invalid signature `{}`", signature),
            AmbiguousEventName(ref name) => write!(
                f,
                "More than one event found for name `{}`, try providing the full signature",
                name
            ),
            InvalidFunctionSignature(ref signature) => {
                write!(f, "Invalid function signature `{}`", signature)
            }
            AmbiguousFunctionName(ref name) => write!(
                f,
                "More than one function found for name `{}`, try providing the full signature",
                name
            ),
            Ethabi(ref err) => write!(f, "Ethabi error: {}", err),
            Io(ref err) => write!(f, "IO error: {}", err),
            Hex(ref err) => write!(f, "Hex parsing error: {}", err),
        }
    }
}

impl From<ethabi::Error> for Error {
    fn from(err: ethabi::Error) -> Self {
        Error::Ethabi(err)
    }
}
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
impl From<rustc_hex::FromHexError> for Error {
    fn from(err: rustc_hex::FromHexError) -> Self {
        Error::Hex(err)
    }
}
