use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Users::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Users::Id)
							.uuid()
							.not_null()
							.extra("DEFAULT uuid_generate_v4()")
							.primary_key(),
					)
					.col(ColumnDef::new(Users::Username).string().not_null())
					.col(ColumnDef::new(Users::Password).string().not_null())
					.col(
						ColumnDef::new(Users::PasswordSalt)
							.uuid()
							.not_null()
							.extra("DEFAULT uuid_generate_v4()"),
					)
					.col(
						ColumnDef::new(Users::TokenSalt)
							.uuid()
							.not_null()
							.extra("DEFAULT uuid_generate_v4()"),
					)
					.col(ColumnDef::new(Users::Cid).uuid().not_null())
					.col(
						ColumnDef::new(Users::Ctime)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.col(ColumnDef::new(Users::Mid).uuid().not_null())
					.col(
						ColumnDef::new(Users::Mtime)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Users::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
enum Users {
	Table,
	Id,
	Username,
	Password,
	PasswordSalt,
	TokenSalt,
	Cid,
	Ctime,
	Mid,
	Mtime,
}
