use serde::Serialize;

use crate::pwd::scheme;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
	PwdWithSchemeParseFail,

	// -- Modules
	Scheme(scheme::Error),
}

// region:    --- From
impl From<scheme::Error> for Error {
	fn from(val: scheme::Error) -> Self {
		Self::Scheme(val)
	}
}
// endregion: --- From

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
