use std::str::FromStr;

use crate::{ctx::Ctx, model::ModelManager};

use modql::filter::{FilterNodes, OpValsBool, OpValsString};

use sea_orm::{
	ActiveModelTrait, Condition, EntityName, EntityTrait, FromQueryResult,
	PaginatorTrait, QueryFilter, QueryOrder, Set,
};

use crate::model::sea_query;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::entity::prelude::Tasks;
use super::entity::tasks::{self, Model};
use super::ListOptions;
use crate::model::{Error, Result};

#[derive(Deserialize, Serialize)]
pub struct Task {
	pub id: Uuid,
	pub project_id: Uuid,

	pub title: String,
	pub done: bool,
}

#[derive(Deserialize)]
pub struct TaskForCreate {
	pub title: String,
	pub project_id: Uuid,
}

#[derive(Deserialize, Default, FromQueryResult)]
pub struct TaskForUpdate {
	pub title: Option<String>,
	pub done: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct TaskFilter {
	// FIXME: uuid filter error
	project_id: Option<OpValsString>,
	title: Option<OpValsString>,
	done: Option<OpValsBool>,
}

impl From<Model> for Task {
	fn from(value: Model) -> Self {
		Self {
			id: value.id,
			project_id: value.project_id,
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
			project_id: Set(task_c.project_id),
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
		list_options: Option<ListOptions>,
	) -> Result<Vec<Task>> {
		let db = mm.db();
		let mut query: sea_orm::prelude::Select<Tasks> = Tasks::find();
		// condition from filter
		if let Some(filter) = filter {
			let cond: Condition = filter.try_into()?;
			query = query.filter(cond);
		}

		if let Some(list_options) = list_options {
			if let Some(order_bys) = list_options.convert_order_by() {
				for (col, order) in order_bys.into_iter() {
					query =
						query.order_by(tasks::Column::from_str(col.as_str())?, order)
				}
			}
			// let total = query.clone().count(db).await?;
			let pagintor =
				query.paginate(db, ListOptions::as_positive_u64(list_options.limit));
			// let total_pages = pagintor.num_pages().await?;
			let ret: Vec<Task> = pagintor
				.fetch_page(ListOptions::as_positive_u64(list_options.offset))
				.await?
				.into_iter()
				.map(|t| t.into())
				.collect();
			return Ok(ret);
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
	use crate::model::{
		project::{ProjectBmc, ProjectForCreate},
		Error,
	};
	use anyhow::Result;
	use modql::filter::OpValString;
	use sea_orm::{
		sea_query::{ConditionExpression, Expr, IntoCondition},
		ColumnTrait,
	};
	use serde_json::json;

	#[tokio::test]
	async fn test_create_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let fix_title = "test_create_ok title";
		let mm = ModelManager::new().await?;
		let project_c = ProjectForCreate {
			name: "test_create_ok project for task ".to_string(),
		};
		let project_id = ProjectBmc::create(&ctx, &mm, project_c).await?;
		let task_c = TaskForCreate {
			title: fix_title.to_string(),
			project_id,
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

		let project_c = ProjectForCreate {
			name: "test_list_all_ok project for task".to_string(),
		};
		let project_id = ProjectBmc::create(&ctx, &mm, project_c).await?;

		let fx_titles = &["test_list_all_ok-task 01", "test_list_all_ok-task 02"];
		for title in fx_titles {
			let id = TaskBmc::create(
				&ctx,
				&mm,
				TaskForCreate {
					title: title.to_string(),
					project_id,
				},
			)
			.await?;
		}

		let tasks = TaskBmc::list(&ctx, &mm, None, None).await?;

		// Check
		let tasks: Vec<Task> = tasks
			.into_iter()
			.filter(|t| t.title.starts_with("test_list_all_ok-task"))
			.collect();
		assert_eq!(tasks.len(), 2, "number of seeded tasks.");

		// -- Clean
		ProjectBmc::delete(&ctx, &mm, project_id).await?;
		Ok(())
	}

	#[tokio::test]
	async fn test_list_by_title_contains_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = ModelManager::new().await?;
		let project_c = ProjectForCreate {
			name: "test_list_by_title_contains_ok project for task".to_string(),
		};
		let project_id = ProjectBmc::create(&ctx, &mm, project_c).await?;
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
					project_id,
				},
			)
			.await?;
		}

		let filter = TaskFilter {
			project_id: Some(project_id.to_string().into()),
			title: Some(
				OpValString::Contains("by_title_contains_ok 02".to_string()).into(),
			),
			..Default::default()
		};
		let mut cond: Condition = filter.try_into()?;
		cond = cond.into_condition().add(
			Expr::col(tasks::Column::ProjectId)
				.eq(project_id)
				.into_condition(),
		);
		println!("{:?}", cond);
		// let tasks = TaskBmc::list(&ctx, &mm, Some(filter), None).await?;

		// // -- Check
		// assert_eq!(tasks.len(), 2);

		// -- Cleanup
		ProjectBmc::delete(&ctx, &mm, project_id).await?;

		Ok(())
	}

	#[tokio::test]
	async fn test_list_with_list_options_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = ModelManager::new().await?;
		let project_c = ProjectForCreate {
			name: "test_list_with_list_options_ok project for task".to_string(),
		};
		let project_id = ProjectBmc::create(&ctx, &mm, project_c).await?;

		let fx_titles = &[
			"test_list_with_list_options_ok 01",
			"test_list_with_list_options_ok 02.1",
			"test_list_with_list_options_ok 02.2",
		];
		for title in fx_titles {
			let id = TaskBmc::create(
				&ctx,
				&mm,
				TaskForCreate {
					title: title.to_string(),
					project_id,
				},
			)
			.await?;
		}
		// -- Exec
		let filter: TaskFilter = serde_json::from_value(json!({
			"title": {"$startsWith": "test_list_with_list_options_ok" }
		}))?;
		let list_options: ListOptions = serde_json::from_value(json! ({
			"offset": 0,
			"limit": 2,
			"order_bys": "!title"
		}))?;
		let tasks =
			TaskBmc::list(&ctx, &mm, Some(filter), Some(list_options)).await?;
		// -- Check
		let titles: Vec<String> =
			tasks.iter().map(|t| t.title.to_string()).collect();
		assert_eq!(titles.len(), 2);
		assert_eq!(
			&titles,
			&[
				"test_list_with_list_options_ok 02.2",
				"test_list_with_list_options_ok 02.1"
			]
		);
		// -- Cleanup
		ProjectBmc::delete(&ctx, &mm, project_id).await?;

		Ok(())
	}

	#[tokio::test]
	async fn test_update_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = ModelManager::new().await?;

		let project_c = ProjectForCreate {
			name: "test_update_ok project for task".to_string(),
		};
		let project_id = ProjectBmc::create(&ctx, &mm, project_c).await?;

		let fx_title = "test_update_ok - task 01";
		let fx_title_new = "test_update_ok - task 01 - new";

		let id = TaskBmc::create(
			&ctx,
			&mm,
			TaskForCreate {
				title: fx_title.to_string(),
				project_id,
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

		// -- Cleanup
		ProjectBmc::delete(&ctx, &mm, project_id).await?;

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
