use sea_orm::sea_query;
use time::serde::rfc3339;
use uuid::Uuid;

pub fn time_to_sea_value(
	json_value: serde_json::Value,
) -> modql::filter::SeaResult<sea_query::Value> {
	Ok(rfc3339::deserialize(json_value)?.into())
}

pub fn uuid_to_sea_value(
	json_value: serde_json::Value,
) -> modql::filter::SeaResult<sea_query::Value> {
	let ret: Uuid = serde_json::from_value(json_value)?;
	Ok(ret.into())
}
