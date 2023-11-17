use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Tasks::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Tasks::Id)
							.uuid()
							.not_null()
							.extra("DEFAULT uuid_generate_v4()")
							.primary_key(),
					)
					.col(ColumnDef::new(Tasks::Title).string().not_null())
					.col(
						ColumnDef::new(Tasks::Done)
							.boolean()
							.not_null()
							.default(false),
					)
					.col(ColumnDef::new(Tasks::Cid).uuid().not_null())
					.col(
						ColumnDef::new(Tasks::Ctime)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.col(ColumnDef::new(Tasks::Mid).uuid().not_null())
					.col(
						ColumnDef::new(Tasks::Mtime)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(Tasks::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
pub enum Tasks {
	Table,
	Id,
	Title,
	Done,
	ProjectId,
	Cid,
	Ctime,
	Mid,
	Mtime,
}
