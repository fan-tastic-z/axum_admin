use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
	pub data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
	pub id: Uuid,
	pub data: D,
}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: Uuid,
}
