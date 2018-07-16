//! Function and event param types.

use std::fmt;
use super::Writer;

/// Function and event param types.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
	/// Address.
	Address,
	/// Bytes.
	Bytes,
	/// Signed integer.
	Int(usize),
	/// Unsigned integer.
	Uint(usize),
	/// Boolean.
	Bool,
	/// String.
	String,
	/// Array of unknown size.
	Array(Box<ParamType>),
	/// Vector of bytes with fixed size.
	FixedBytes(usize),
	/// Array with fixed size.
	FixedArray(Box<ParamType>, usize),
}

impl fmt::Display for ParamType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", Writer::write(self))
	}
}

impl ParamType {
    /// returns whether a zero length byte slice (`0x`) is
    /// a valid encoded form of this param type
    pub fn is_empty_bytes_valid_encoding(&self) -> bool {
        match self {
            ParamType::FixedBytes(len) => *len == 0,
            ParamType::FixedArray(_, len) => *len == 0,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
	use ParamType;

	#[test]
	fn test_param_type_display() {
		assert_eq!(format!("{}", ParamType::Address), "address".to_owned());
		assert_eq!(format!("{}", ParamType::Bytes), "bytes".to_owned());
		assert_eq!(format!("{}", ParamType::FixedBytes(32)), "bytes32".to_owned());
		assert_eq!(format!("{}", ParamType::Uint(256)), "uint256".to_owned());
		assert_eq!(format!("{}", ParamType::Int(64)), "int64".to_owned());
		assert_eq!(format!("{}", ParamType::Bool), "bool".to_owned());
		assert_eq!(format!("{}", ParamType::String), "string".to_owned());
		assert_eq!(format!("{}", ParamType::Array(Box::new(ParamType::Bool))), "bool[]".to_owned());
		assert_eq!(format!("{}", ParamType::FixedArray(Box::new(ParamType::String), 2)), "string[2]".to_owned());
		assert_eq!(format!("{}", ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 2)), "bool[][2]".to_owned());
	}
}
