use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use uuid::Uuid;

use crate::model::store;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
	Store(store::Error),
	SeaORM(#[serde_as(as = "DisplayFromStr")] sea_orm::DbErr),
	EntityNotFound { entity: &'static str, id: Uuid },
}

// region:    --- Froms
impl From<store::Error> for Error {
	fn from(val: store::Error) -> Self {
		Self::Store(val)
	}
}

impl From<sea_orm::DbErr> for Error {
	fn from(val: sea_orm::DbErr) -> Self {
		Self::SeaORM(val)
	}
}
// endregion: --- Froms

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
