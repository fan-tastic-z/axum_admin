use crate::{ctx::Ctx, model::ModelManager};

use modql::filter::{FilterNodes, OpValsBool, OpValsString};

use sea_orm::{
	ActiveModelTrait, Condition, EntityName, EntityTrait, FromQueryResult,
	QueryFilter, Set,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::entity::prelude::Tasks;
use super::entity::tasks::{self, Model};
use crate::model::{Error, Result};

#[derive(Deserialize, Serialize)]
pub struct Task {
	pub id: Uuid,
	pub title: String,
	pub done: bool,
}

#[derive(Deserialize)]
pub struct TaskForCreate {
	pub title: String,
}

#[derive(Deserialize, Default, FromQueryResult)]
pub struct TaskForUpdate {
	pub title: Option<String>,
	pub done: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct TaskFilter {
	title: Option<OpValsString>,
	done: Option<OpValsBool>,
}

impl From<Model> for Task {
	fn from(value: Model) -> Self {
		Self {
			id: value.id,
			title: value.title,
			done: value.done,
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

	pub async fn list(
		_ctx: &Ctx,
		mm: &ModelManager,
		filter: Option<TaskFilter>,
	) -> Result<Vec<Task>> {
		let db = mm.db();
		let mut query = Tasks::find();
		if let Some(filter) = filter {
			let cond: Condition = filter.try_into()?;
			query = query.filter(cond);
		}
		let ret = query
			.clone()
			.all(db)
			.await?
			.into_iter()
			.map(|t| t.into())
			.collect();
		Ok(ret)
	}

	pub async fn update(
		_ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		task_u: TaskForUpdate,
	) -> Result<()> {
		let db = mm.db();
		let table_name = Tasks::table_name(&Tasks);
		let mut tasks_active_model: tasks::ActiveModel = Tasks::find_by_id(id)
			.one(db)
			.await?
			.ok_or(Error::EntityNotFound {
				entity: table_name,
				id,
			})?
			.into();
		if let Some(title) = task_u.title {
			tasks_active_model.title = Set(title);
		}
		if let Some(done) = task_u.done {
			tasks_active_model.done = Set(done);
		}
		tasks_active_model.update(db).await?;
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
	use modql::filter::OpValString;

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
	async fn test_list_all_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = ModelManager::new().await?;

		let fx_titles = &["test_list_all_ok-task 01", "test_list_all_ok-task 02"];
		for title in fx_titles {
			let id = TaskBmc::create(
				&ctx,
				&mm,
				TaskForCreate {
					title: title.to_string(),
				},
			)
			.await?;
		}
		let tasks = TaskBmc::list(&ctx, &mm, None).await?;

		// Check
		let tasks: Vec<Task> = tasks
			.into_iter()
			.filter(|t| t.title.starts_with("test_list_all_ok-task"))
			.collect();
		assert_eq!(tasks.len(), 2, "number of seeded tasks.");

		// -- Clean
		for task in tasks.iter() {
			TaskBmc::delete(&ctx, &mm, task.id).await?;
		}
		Ok(())
	}

	#[tokio::test]
	async fn test_list_by_title_contains_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = ModelManager::new().await?;
		let fx_titles = &[
			"test_list_by_title_contains_ok 01",
			"test_list_by_title_contains_ok 02.1",
			"test_list_by_title_contains_ok 02.2",
		];
		for title in fx_titles {
			let id = TaskBmc::create(
				&ctx,
				&mm,
				TaskForCreate {
					title: title.to_string(),
				},
			)
			.await?;
		}
		let filter = TaskFilter {
			title: Some(
				OpValString::Contains("by_title_contains_ok 02".to_string()).into(),
			),
			..Default::default()
		};
		let tasks = TaskBmc::list(&ctx, &mm, Some(filter)).await?;

		// -- Check
		assert_eq!(tasks.len(), 2);

		// -- Cleanup
		// Will delete associate tasks
		for task in tasks {
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
				title: Some(fx_title_new.to_string()),
				..Default::default()
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
