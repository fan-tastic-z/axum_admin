use modql::filter::ListOptions;
use sea_orm::prelude::Select;
use sea_orm::{sea_query, EntityTrait, QuerySelect};

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

pub fn apply_to_sea_query<E>(
	mut select_entity: Select<E>,
	list_options: &ListOptions,
) -> Select<E>
where
	E: EntityTrait,
{
	fn as_positive_u64(num: i64) -> u64 {
		if num < 0 {
			0
		} else {
			num as u64
		}
	}
	if let Some(limit) = list_options.limit {
		select_entity = select_entity.limit(as_positive_u64(limit)); // Note: Negative == 0
	}

	if let Some(offset) = list_options.offset {
		select_entity = select_entity.offset(as_positive_u64(offset)); // Note: Negative == 0
	}
	select_entity
}
