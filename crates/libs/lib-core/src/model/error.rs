use derive_more::From;
use lib_auth::pwd;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use uuid::Uuid;

use crate::model::store;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
	#[from]
	Store(store::Error),
	#[from]
	SeaORM(#[serde_as(as = "DisplayFromStr")] sea_orm::DbErr),
	EntityNotFound {
		entity: &'static str,
		id: Uuid,
	},
	#[from]
	Pwd(pwd::Error),
	#[from]
	ModqlIntoSea(#[serde_as(as = "DisplayFromStr")] modql::filter::IntoSeaError),
	#[from]
	ColumnFromStrErr(
		#[serde_as(as = "DisplayFromStr")] sea_orm::prelude::ColumnFromStrErr,
	),
	#[from]
	ChronoParseError(#[serde_as(as = "DisplayFromStr")] chrono::ParseError),

	ListLimitOverMax {
		max: i64,
		actual: i64,
	},
}

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
