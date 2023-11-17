use chrono::Local;
use lib_core::{
	model::entity::users::{self, Entity},
	pwd::{self, ContentToHash},
};
use sea_orm_migration::{
	prelude::*,
	sea_orm::{
		ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set,
		TransactionTrait,
	},
};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let db = manager.get_connection();
		let transaction = db.begin().await?;
		let u: users::ActiveModel = users::ActiveModel {
			username: Set("demo1".to_string()),
			password: Set("".to_string()),
			cid: Set(Uuid::default()),
			ctime: Set(Local::now().fixed_offset()),
			mid: Set(Uuid::default()),
			mtime: Set(Local::now().fixed_offset()),
			..Default::default()
		}
		.insert(&transaction)
		.await?
		.into();

		let pwd = pwd::hash_pwd(&ContentToHash {
			content: "welcome".to_string(),
			salt: u.password_salt.clone().unwrap(),
		})
		.unwrap();

		users::ActiveModel {
			password: Set(pwd),
			..u
		}
		.update(&transaction)
		.await?;

		transaction.commit().await?;
		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let db = manager.get_connection();

		Entity::delete_many()
			.filter(users::Column::Username.eq("demo1"))
			.exec(db)
			.await?;

		Ok(())
	}
}
