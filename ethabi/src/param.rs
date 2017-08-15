use spec::{self, ParamType};

/// Function param.
#[derive(Clone, Debug, PartialEq)]
pub struct Param {
	/// Param name.
	pub name: String,
	/// Param kind.
	pub kind: ParamType,
}

impl From<spec::Param> for Param {
	fn from(param: spec::Param) -> Self {
		Param {
			name: param.name,
			kind: param.kind,
		}
	}
}

