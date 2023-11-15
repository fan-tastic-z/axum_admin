use std::str::FromStr;

use modql::filter::FilterNodes;
use modql::filter::OpValsString;
use sea_orm::ActiveModelTrait;
use sea_orm::Condition;
use sea_orm::EntityName;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ctx::Ctx;
use crate::model::entity::prelude::Projects;
use crate::model::entity::projects;
use crate::model::sea_query;
use crate::model::ListOptions;
use crate::model::ModelManager;
use crate::model::{Error, Result};

use crate::model::entity::projects::Model;

// region:    --- Project Types
#[derive(Debug, Clone, Serialize)]
pub struct Project {
	pub id: Uuid,
	pub name: String,
	pub owner_id: Uuid,
}

impl From<Model> for Project {
	fn from(val: Model) -> Self {
		Self {
			id: val.id,
			name: val.name,
			owner_id: val.owner_id,
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
	name: Option<OpValsString>,
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
		let project = projects::ActiveModel {
			name: Set(project_c.name),
			owner_id: Set(project_c.owner_id),
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
		filter: Option<ProjectFilter>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<Project>> {
		let db = mm.db();
		let mut query = Projects::find();
		if let Some(filter) = filter {
			let cond: Condition = filter.try_into()?;
			query = query.filter(cond);
		}

		if let Some(list_options) = list_options {
			if let Some(order_bys) = list_options.convert_order_by() {
				for (col, order) in order_bys.into_iter() {
					query = query
						.order_by(projects::Column::from_str(col.as_str())?, order)
				}
			}
			// let total = query.clone().count(db).await?;
			let pagintor =
				query.paginate(db, ListOptions::as_positive_u64(list_options.limit));
			// let total_pages = pagintor.num_pages().await?;
			let ret: Vec<Project> = pagintor
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
