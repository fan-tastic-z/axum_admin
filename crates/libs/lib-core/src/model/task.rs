use crate::{ctx::Ctx, model::ModelManager};
use sea_orm::{ActiveModelTrait, EntityName, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::entity::prelude::Tasks;
use super::entity::tasks::{self, Model};
use crate::model::{Error, Result};

#[derive(Deserialize, Serialize)]
pub struct Task {
	pub id: Uuid,
	pub title: String,
}

#[derive(Deserialize)]
pub struct TaskForCreate {
	pub title: String,
}

#[derive(Deserialize)]
pub struct TaskForUpdate {
	pub title: String,
}

impl From<Model> for Task {
	fn from(value: Model) -> Self {
		Self {
			id: value.id,
			title: value.title,
		}
	}
}

pub struct TaskBmc;

impl TaskBmc {
	pub async fn create(
		_ctx: &Ctx,
		mm: &ModelManager,
		task_c: TaskForCreate,
	) -> Result<Uuid> {
		let db = mm.db();
		let task = tasks::ActiveModel {
			title: Set(task_c.title),
			..Default::default()
		};
		let ret = Tasks::insert(task).exec(db).await?;
		Ok(ret.last_insert_id)
	}

	pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<Model> {
		let db = mm.db();
		let table_name = Tasks::table_name(&Tasks);

		let entity =
			Tasks::find_by_id(id)
				.one(db)
				.await?
				.ok_or(Error::EntityNotFound {
					entity: table_name,
					id,
				})?;
		Ok(entity)
	}

	pub async fn list(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
		let db = mm.db();
		let entities = Tasks::find()
			.all(db)
			.await?
			.into_iter()
			.map(|t| t.into())
			.collect();
		Ok(entities)
	}

	pub async fn update(
		_ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		task_u: TaskForUpdate,
	) -> Result<()> {
		let db = mm.db();
		let table_name = Tasks::table_name(&Tasks);
		let entity =
			Tasks::find_by_id(id)
				.one(db)
				.await?
				.ok_or(Error::EntityNotFound {
					entity: table_name,
					id,
				})?;
		let act = tasks::ActiveModel {
			title: Set(task_u.title),
			..entity.into()
		};
		act.update(db).await?;
		Ok(())
	}

	pub async fn delete(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		let db = mm.db();
		let table_name = Tasks::table_name(&Tasks);
		let dr = Tasks::delete_by_id(id).exec(db).await?;
		if dr.rows_affected == 0 {
			return Err(Error::EntityNotFound {
				entity: table_name,
				id,
			});
		}
		Ok(())
	}
}

// region:    --- TestBmc
#[cfg(test)]
mod tests {
	#![allow(unused)]
	use super::*;
	use crate::model::Error;
	use anyhow::Result;

	#[tokio::test]
	async fn test_create_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let fix_title = "test_create_ok title";
		let mm = ModelManager::new().await?;

		let task_c = TaskForCreate {
			title: fix_title.to_string(),
		};
		let id = TaskBmc::create(&ctx, &mm, task_c).await?;
		// -- check
		let task: Task = TaskBmc::get(&ctx, &mm, id.clone()).await?.into();
		assert_eq!(task.title, fix_title);

		// -- clean
		TaskBmc::delete(&ctx, &mm, id).await?;

		Ok(())
	}

	#[tokio::test]
	async fn test_get_err_not_found() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let fx_id = Uuid::new_v4();
		let mm = ModelManager::new().await?;
		let res = TaskBmc::get(&ctx, &mm, fx_id).await;

		// -- Check
		assert!(
			matches!(
				res,
				Err(Error::EntityNotFound {
					entity: "tasks",
					id: fx_id
				})
			),
			"EntityNotFound not matching"
		);
		Ok(())
	}

	#[tokio::test]
	async fn test_list_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = ModelManager::new().await?;

		let fx_titles = &["test_list_ok-task 01", "test_list_ok-task 02"];
		let mut create_tasks = Vec::new();
		for title in fx_titles {
			let id = TaskBmc::create(
				&ctx,
				&mm,
				TaskForCreate {
					title: title.to_string(),
				},
			)
			.await?;
			let task: Task = TaskBmc::get(&ctx, &mm, id).await?.into();
			create_tasks.push(task)
		}
		// check
		let tasks: Vec<Task> = create_tasks
			.into_iter()
			.filter(|t| t.title.starts_with("test_list_ok-task"))
			.collect();
		assert_eq!(tasks.len(), 2, "number of seeded tasks.");

		// -- Clean
		for task in tasks.iter() {
			TaskBmc::delete(&ctx, &mm, task.id).await?;
		}
		Ok(())
	}

	#[tokio::test]
	async fn test_update_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = ModelManager::new().await?;
		let fx_title = "test_update_ok - task 01";
		let fx_title_new = "test_update_ok - task 01 - new";

		let id = TaskBmc::create(
			&ctx,
			&mm,
			TaskForCreate {
				title: fx_title.to_string(),
			},
		)
		.await?;

		let task: Task = TaskBmc::get(&ctx, &mm, id).await?.into();

		TaskBmc::update(
			&ctx,
			&mm,
			id,
			TaskForUpdate {
				title: fx_title_new.to_string(),
			},
		)
		.await?;

		// -- Check
		let task: Task = TaskBmc::get(&ctx, &mm, task.id).await?.into();
		assert_eq!(task.title, fx_title_new);

		Ok(())
	}

	#[tokio::test]
	async fn test_delete_err_not_found() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = ModelManager::new().await?;
		let fix_id = Uuid::new_v4();

		let res = TaskBmc::delete(&ctx, &mm, fix_id).await;

		// -- Check
		assert!(
			matches!(
				res,
				Err(Error::EntityNotFound {
					entity: "tasks",
					id: fix_id
				})
			),
			"EntityNotFound not matching"
		);

		Ok(())
	}
}

// endregion: --- TestBmc
