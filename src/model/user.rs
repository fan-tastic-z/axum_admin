use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use super::ModelManager;
use crate::model::entity::{prelude::Users, users};
use crate::model::Result;

pub struct UserBmc;

impl UserBmc {
	pub async fn first_by_username(
		mm: &ModelManager,
		username: &str,
	) -> Result<Option<users::Model>> {
		let db = mm.db();
		let user = Users::find()
			.filter(users::Column::Username.eq(username))
			.one(db)
			.await?;
		Ok(user)
	}
}
