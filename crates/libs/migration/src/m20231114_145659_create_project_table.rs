use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Projects::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Projects::Id)
							.uuid()
							.not_null()
							.extra("DEFAULT uuid_generate_v4()")
							.primary_key(),
					)
					.col(ColumnDef::new(Projects::OwnerId).uuid().not_null())
					.col(ColumnDef::new(Projects::Name).string().not_null())
					.col(ColumnDef::new(Projects::Cid).uuid().not_null())
					.col(
						ColumnDef::new(Projects::Ctime)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.col(ColumnDef::new(Projects::Mid).uuid().not_null())
					.col(
						ColumnDef::new(Projects::Mtime)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Projects::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum Projects {
	Table,
	Id,
	OwnerId,
	Name,
	Cid,
	Ctime,
	Mid,
	Mtime,
}
