use crate::web::Result;
use lib_core::{
	ctx::Ctx,
	model::{
		project::{
			Project, ProjectBmc, ProjectFilter, ProjectForCreate, ProjectForUpdate,
		},
		ModelManager,
	},
};

use crate::web::rpc::params::{
	ParamsForCreate, ParamsForUpdate, ParamsIded, ParamsList,
};

pub async fn create_project(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForCreate<ProjectForCreate>,
) -> Result<Project> {
	let ParamsForCreate { data } = params;

	let id = ProjectBmc::create(&ctx, &mm, data).await?;
	let project = ProjectBmc::get(&ctx, &mm, id).await?;

	Ok(project.into())
}

pub async fn list_projects(
	ctx: Ctx,
	mm: ModelManager,
	params: Option<ParamsList<ProjectFilter>>,
) -> Result<Vec<Project>> {
	let (filter, list_options) = params.map(|p| (p.filter, p.list_options)).unzip();
	let projects =
		ProjectBmc::list(&ctx, &mm, filter.flatten(), list_options.flatten())
			.await?;

	Ok(projects)
}

pub async fn update_project(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForUpdate<ProjectForUpdate>,
) -> Result<Project> {
	let ParamsForUpdate { id, data } = params;

	ProjectBmc::update(&ctx, &mm, id, data).await?;

	let project = ProjectBmc::get(&ctx, &mm, id).await?;

	Ok(project.into())
}

pub async fn delete_project(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsIded,
) -> Result<Project> {
	let ParamsIded { id } = params;

	let project = ProjectBmc::get(&ctx, &mm, id).await?;
	ProjectBmc::delete(&ctx, &mm, id).await?;

	Ok(project.into())
}
