use std::str::FromStr;

use crate::{ctx::Ctx, model::ModelManager};

use chrono::{DateTime, FixedOffset};
use lib_base::time::date_time_with_zone;
use modql::filter::{
	FilterGroups, FilterNodes, OpValsBool, OpValsString, OpValsValue,
};

use crate::model::modql_utils::{time_to_sea_value, uuid_to_sea_value};
use sea_orm::{
	ActiveModelTrait, Condition, EntityName, EntityTrait, FromQueryResult,
	QueryFilter, QueryOrder, Set,
};

use crate::model::sea_query;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::entity::prelude::Tasks;
use super::entity::tasks::{self, Model};
use super::modql_utils::apply_to_sea_query;
use super::{compute_list_options, ListOptions};
use crate::model::{Error, Result};

#[derive(Deserialize, Serialize, Debug)]
pub struct Task {
	pub id: Uuid,
	pub project_id: Uuid,

	pub title: String,
	pub done: bool,

	pub cid: Uuid,
	pub ctime: DateTime<FixedOffset>,
	pub mid: Uuid,
	pub mtime: DateTime<FixedOffset>,
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
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	id: Option<OpValsValue>,
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	project_id: Option<OpValsValue>,
	title: Option<OpValsString>,
	done: Option<OpValsBool>,

	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	cid: Option<OpValsValue>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	ctime: Option<OpValsValue>,
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	mid: Option<OpValsValue>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	mtime: Option<OpValsValue>,
}

impl From<Model> for Task {
	fn from(val: Model) -> Self {
		Self {
			id: val.id,
			project_id: val.project_id,
			title: val.title,
			done: val.done,

			cid: val.cid,
			ctime: val.ctime.into(),
			mid: val.mid,
			mtime: val.mtime.into(),
		}
	}
}

pub struct TaskBmc;

impl TaskBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		task_c: TaskForCreate,
	) -> Result<Uuid> {
		let db = mm.db();
		let dt = DateTime::parse_from_rfc3339(&date_time_with_zone().to_rfc3339())?;
		let task = tasks::ActiveModel {
			title: Set(task_c.title),
			project_id: Set(task_c.project_id),
			cid: Set(ctx.user_id()),
			ctime: Set(dt),
			mid: Set(ctx.user_id()),
			mtime: Set(dt),
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
		filter: Option<Vec<TaskFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<Task>> {
		let db = mm.db();
		let mut query: sea_orm::prelude::Select<Tasks> = Tasks::find();
		// condition from filter
		if let Some(filter) = filter {
			let filters: FilterGroups = filter.into();
			let cond: Condition = filters.try_into()?;
			query = query.filter(cond);
		}

		let list_options = compute_list_options(list_options)?;
		query = apply_to_sea_query(query, &list_options);
		if let Some(order_bys) = list_options.order_bys {
			for order in order_bys.into_iter() {
				match order {
					modql::filter::OrderBy::Asc(col) => {
						query = query
							.order_by_asc(tasks::Column::from_str(col.as_str())?);
					}
					modql::filter::OrderBy::Desc(col) => {
						query = query
							.order_by_desc(tasks::Column::from_str(col.as_str())?);
					}
				}
			}
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
	use std::time::Duration;

	use super::*;
	use crate::model::{
		project::{ProjectBmc, ProjectForCreate},
		Error,
	};
	use anyhow::Result;
	use lib_base::time::{format_time, now_utc};
	use modql::filter::OpValString;
	use sea_orm::{
		sea_query::{ConditionExpression, Expr, IntoCondition},
		ColumnTrait,
	};
	use serde_json::json;
	use tokio::time::sleep;

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
		let filter = serde_json::from_value(json!({
			"title": {"$contains": "by_title_contains_ok 02"},
			"project_id": {"$eq": project_id},
		}))?;
		let tasks = TaskBmc::list(&ctx, &mm, Some(vec![filter]), None).await?;
		// -- Check
		assert_eq!(tasks.len(), 2);

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
			TaskBmc::list(&ctx, &mm, Some(vec![filter]), Some(list_options)).await?;

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
	async fn test_list_by_ctime_ok() -> Result<()> {
		let ctx = Ctx::root_ctx();
		let mm = ModelManager::new().await?;
		let project_c = ProjectForCreate {
			name: "project for tasks test_list_by_ctime_ok".to_string(),
		};
		let project_id = ProjectBmc::create(&ctx, &mm, project_c).await?;
		let fx_titles_01 = &[
			"test_list_by_ctime_ok 01.1",
			"test_list_by_ctime_ok 01.2",
			"test_list_by_ctime_ok 01.3",
		];
		for title in fx_titles_01 {
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
		let time_marker = format_time(now_utc());
		sleep(Duration::from_millis(300)).await;
		let fx_titles_02 =
			&["test_list_by_ctime_ok 02.1", "test_list_by_ctime_ok 02.2"];
		for title in fx_titles_02 {
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
		let filter_json = json! ({
			"ctime": {"$gt": time_marker}, // time in Rfc3339
		});
		let filter = vec![serde_json::from_value(filter_json)?];
		let tasks = TaskBmc::list(&ctx, &mm, Some(filter), None).await?;
		// -- Check
		let titles: Vec<String> = tasks.into_iter().map(|t| t.title).collect();
		assert_eq!(titles.len(), 2);
		assert_eq!(&titles, fx_titles_02);

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
