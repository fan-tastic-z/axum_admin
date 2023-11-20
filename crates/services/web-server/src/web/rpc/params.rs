use lib_core::model::ListOptions;
use serde::{de::DeserializeOwned, Deserialize};
use uuid::Uuid;

use super::infra::IntoHandlerParams;

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
	pub data: D,
}

impl<D> IntoHandlerParams for ParamsForCreate<D> where D: DeserializeOwned + Send {}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
	pub id: Uuid,
	pub data: D,
}

impl<D> IntoHandlerParams for ParamsForUpdate<D> where D: DeserializeOwned + Send {}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: Uuid,
}

impl IntoHandlerParams for ParamsIded {}

#[derive(Deserialize)]
pub struct ParamsList<F> {
	pub filter: Option<F>,
	pub list_options: Option<ListOptions>,
}

impl<F> IntoHandlerParams for ParamsList<F> where F: DeserializeOwned + Send {}

impl<F> IntoHandlerParams for Option<ParamsList<F>>
where
	F: DeserializeOwned + Send,
{
	fn into_handler_params(
		value: Option<sea_orm::prelude::Json>,
	) -> crate::web::Result<Self> {
		match value {
			Some(value) => Ok(serde_json::from_value(value)?),
			None => Ok(None),
		}
	}
}
