use std::str::FromStr;

use chrono::DateTime;
use chrono::FixedOffset;

use lib_base::time::date_time_with_zone;

use modql::filter::FilterGroups;
use modql::filter::FilterNodes;
use modql::filter::ListOptions;
use modql::filter::OpValsString;

use modql::filter::OpValsValue;
use sea_orm::ActiveModelTrait;
use sea_orm::Condition;
use sea_orm::EntityName;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::ctx::Ctx;
use crate::model::entity::prelude::Projects;
use crate::model::entity::projects;
use crate::model::modql_utils::{time_to_sea_value, uuid_to_sea_value};
use crate::model::sea_query;
// use crate::model::ListOptions;
use crate::model::ModelManager;
use crate::model::{Error, Result};

use crate::model::entity::projects::Model;

use super::compute_list_options;
use super::modql_utils::apply_to_sea_query;

// region:    --- Project Types
#[derive(Debug, Clone, Serialize)]
pub struct Project {
	pub id: Uuid,
	pub name: String,
	pub owner_id: Uuid,

	pub cid: Uuid,
	pub ctime: DateTime<FixedOffset>,
	pub mid: Uuid,
	pub mtime: DateTime<FixedOffset>,
}

impl From<Model> for Project {
	fn from(val: Model) -> Self {
		Self {
			id: val.id,
			name: val.name,
			owner_id: val.owner_id,
			cid: val.cid,
			ctime: val.ctime.into(),
			mid: val.mid,
			mtime: val.mtime.into(),
		}
	}
}

#[derive(Deserialize)]
pub struct ProjectForCreate {
	pub name: String,
}

struct ProjectForCreateInner {
	pub name: String,
	pub owner_id: Uuid,
}

#[derive(FilterNodes, Default, Deserialize)]
pub struct ProjectFilter {
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	id: Option<OpValsValue>,
	name: Option<OpValsString>,

	#[modql(to_sea_value_fn = "time_to_sea_value")]
	ctime: Option<OpValsValue>,
	#[modql(to_sea_value_fn = "uuid_to_sea_value")]
	mid: Option<OpValsValue>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	mtime: Option<OpValsValue>,
}

#[derive(Deserialize)]
pub struct ProjectForUpdate {
	pub name: Option<String>,
	pub owner_id: Option<i64>,
}
// endregion: --- Project Types

// region:    --- ProjectBmc
pub struct ProjectBmc;

impl ProjectBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		project_c: ProjectForCreate,
	) -> Result<Uuid> {
		let project_c = ProjectForCreateInner {
			name: project_c.name,
			owner_id: ctx.user_id(),
		};
		let db = mm.db();
		let dt = DateTime::parse_from_rfc3339(&date_time_with_zone().to_rfc3339())?;
		let project = projects::ActiveModel {
			name: Set(project_c.name),
			owner_id: Set(project_c.owner_id),
			cid: Set(ctx.user_id()),
			ctime: Set(dt),
			mid: Set(ctx.user_id()),
			mtime: Set(dt),
			..Default::default()
		};
		let ret = Projects::insert(project).exec(db).await?;
		Ok(ret.last_insert_id)
	}

	pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<Model> {
		let db = mm.db();
		// now vscode doesn't auto `use sea_orm::EntityName;`
		let table_name = Projects::table_name(&Projects);
		let entity = Projects::find_by_id(id).one(db).await?.ok_or(
			Error::EntityNotFound {
				entity: table_name,
				id,
			},
		)?;
		Ok(entity)
	}

	pub async fn list(
		_ctx: &Ctx,
		mm: &ModelManager,
		filter: Option<Vec<ProjectFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<Project>> {
		let db = mm.db();
		let mut query = Projects::find();
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
							.order_by_asc(projects::Column::from_str(col.as_str())?);
					}
					modql::filter::OrderBy::Desc(col) => {
						query = query.order_by_desc(projects::Column::from_str(
							col.as_str(),
						)?);
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
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		project_u: ProjectForUpdate,
	) -> Result<()> {
		let db = mm.db();
		let table_name = Projects::table_name(&Projects);
		let mut projects_active_model: projects::ActiveModel =
			Projects::find_by_id(id)
				.one(db)
				.await?
				.ok_or(Error::EntityNotFound {
					entity: table_name,
					id,
				})?
				.into();
		if let Some(name) = project_u.name {
			projects_active_model.name = Set(name)
		}
		let dt = DateTime::parse_from_rfc3339(&date_time_with_zone().to_rfc3339())?;
		projects_active_model.mid = Set(ctx.user_id());
		projects_active_model.mtime = Set(dt);
		projects_active_model.update(db).await?;
		Ok(())
	}

	pub async fn delete(_ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		let db = mm.db();
		let table_name = Projects::table_name(&Projects);
		let dr = Projects::delete_by_id(id).exec(db).await?;
		if dr.rows_affected == 0 {
			return Err(Error::EntityNotFound {
				entity: table_name,
				id,
			});
		}
		Ok(())
	}
}

// endregion: --- ProjectBmcpub struct ProjectBmc;
