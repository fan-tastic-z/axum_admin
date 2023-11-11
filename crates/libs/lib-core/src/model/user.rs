use sea_orm::{
	ActiveModelTrait, ColumnTrait, EntityName, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use super::ModelManager;
use crate::ctx::Ctx;
use crate::model::entity::users::Model;
use crate::model::entity::{prelude::Users, users};
use crate::model::{Error, Result};
use crate::pwd::{self, ContentToHash};
pub struct UserBmc;

impl UserBmc {
	pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<Model> {
		let db = mm.db();
		// if need to get table name , need to use sea_orm::EntityName
		let table_name = Users::table_name(&Users);
		let entity =
			Users::find_by_id(id)
				.one(db)
				.await?
				.ok_or(Error::EntityNotFound {
					entity: table_name,
					id,
				})?;
		Ok(entity)
	}

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

	pub async fn update_pwd(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		pwd_clear: &str,
	) -> Result<()> {
		let db = mm.db();
		let table_name = Users::table_name(&Users);

		let user = Self::get(ctx, mm, id).await?;
		let pwd = pwd::hash_pwd(&ContentToHash {
			content: pwd_clear.to_string(),
			salt: user.password_salt,
		})?;
		let entity =
			Users::find_by_id(id)
				.one(db)
				.await?
				.ok_or(Error::EntityNotFound {
					entity: table_name,
					id,
				})?;
		users::ActiveModel {
			password: Set(pwd),
			..entity.into()
		}
		.update(db)
		.await?;
		Ok(())
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
