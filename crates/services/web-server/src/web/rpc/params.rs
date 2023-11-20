use lib_core::model::ListOptions;
use serde::{de::DeserializeOwned, Deserialize};
use uuid::Uuid;

use crate::web::rpc::router::{IntoDefaultParams, IntoParams};

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
	pub data: D,
}

impl<D> IntoParams for ParamsForCreate<D> where D: DeserializeOwned + Send {}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
	pub id: Uuid,
	pub data: D,
}

impl<D> IntoParams for ParamsForUpdate<D> where D: DeserializeOwned + Send {}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: Uuid,
}

impl IntoParams for ParamsIded {}

#[derive(Deserialize, Default)]
pub struct ParamsList<F> {
	pub filter: Option<F>,
	pub list_options: Option<ListOptions>,
}

impl<D> IntoDefaultParams for ParamsList<D> where D: DeserializeOwned + Send + Default
{}
