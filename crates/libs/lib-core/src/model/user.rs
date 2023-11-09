use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use super::entity::users::Model;
use super::ModelManager;
use crate::ctx::Ctx;
use crate::model::entity::{prelude::Users, users};
use crate::model::Result;

pub struct UserBmc;

impl UserBmc {
	pub async fn first_by_username(
		_ctx: &Ctx,
		mm: &ModelManager,
		username: &str,
	) -> Result<Option<Model>> {
		let db = mm.db();
		let user = Users::find()
			.filter(users::Column::Username.eq(username))
			.one(db)
			.await?;
		Ok(user.into())
	}
}

#[derive(Debug, Clone)]
pub struct UserForLogin {
	pub id: Uuid,
	pub username: String,

	pub password: Option<String>, // hashed with #_scheme_id_#....
	pub password_salt: Uuid,
	pub token_salt: Uuid,
}

impl From<Model> for UserForLogin {
	fn from(value: Model) -> Self {
		Self {
			id: value.id,
			username: value.username,
			password: Some(value.password),
			password_salt: value.password_salt,
			token_salt: value.token_salt,
		}
	}
}

#[derive(Debug, Clone)]
pub struct UserForAuth {
	pub id: Uuid,
	pub username: String,

	// -- token info
	pub token_salt: Uuid,
}

impl From<Model> for UserForAuth {
	fn from(value: Model) -> Self {
		Self {
			id: value.id,
			username: value.username,
			token_salt: value.token_salt,
		}
	}
}
