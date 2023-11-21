pub mod entity;
mod error;
pub mod modql_utils;
mod order_by;
pub mod project;
mod store;
pub mod task;
pub mod user;
pub use order_by::*;

use serde::Deserialize;

pub use self::error::{Error, Result};
use self::store::{new_db_pool, Db};
use sea_orm::sea_query;

const LIST_LIMIT_DEFAULT: i64 = 1000;
const LIST_LIMIT_MAX: i64 = 5000;

#[derive(Clone)]
pub struct ModelManager {
	db: Db,
}

impl ModelManager {
	pub async fn new() -> Result<Self> {
		let db = new_db_pool().await?;
		Ok(ModelManager { db })
	}

	pub fn db(&self) -> &Db {
		&self.db
	}
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct ListOptions {
	pub limit: Option<i64>,
	pub offset: Option<i64>,
	pub order_bys: Option<OrderBys>,
}

impl From<OrderBys> for ListOptions {
	fn from(val: OrderBys) -> Self {
		Self {
			order_bys: Some(val),
			..Default::default()
		}
	}
}

impl From<OrderBys> for Option<ListOptions> {
	fn from(val: OrderBys) -> Self {
		Some(ListOptions {
			order_bys: Some(val),
			..Default::default()
		})
	}
}

impl From<OrderBy> for ListOptions {
	fn from(val: OrderBy) -> Self {
		Self {
			order_bys: Some(OrderBys::from(val)),
			..Default::default()
		}
	}
}

impl From<OrderBy> for Option<ListOptions> {
	fn from(val: OrderBy) -> Self {
		Some(ListOptions {
			order_bys: Some(OrderBys::from(val)),
			..Default::default()
		})
	}
}

impl ListOptions {
	fn as_positive_u64(num: i64) -> u64 {
		if num < 0 {
			0
		} else {
			num as u64
		}
	}
	pub fn convert_order_by(
		&self,
	) -> Option<impl Iterator<Item = (String, sea_query::Order)>> {
		if let Some(order_bys) = &self.order_bys {
			return Some(order_bys.clone().into_sea_orm_col_order_iter());
		}
		return None;
	}
}

pub fn compute_list_options(
	list_options: Option<ListOptions>,
) -> Result<ListOptions> {
	if let Some(mut list_options) = list_options {
		// Validate the limit.
		if let Some(limit) = list_options.limit {
			if limit > LIST_LIMIT_MAX {
				return Err(Error::ListLimitOverMax {
					max: LIST_LIMIT_MAX,
					actual: limit,
				});
			}
		}
		// Set the default limit if no limit
		else {
			list_options.limit = Some(LIST_LIMIT_DEFAULT);
		}
		Ok(list_options)
	}
	// When None, return default
	else {
		Ok(ListOptions {
			limit: Some(LIST_LIMIT_DEFAULT),
			offset: None,
			order_bys: Some("id".into()),
		})
	}
}

// region:    --- TestBmc
#[cfg(test)]
mod tests {
	#![allow(unused)]
	use super::*;
	use anyhow::Result;
	use serde_json::json;

	#[test]
	fn test_order_by() -> Result<()> {
		println!("123");
		let list_options: ListOptions = serde_json::from_value(json! ({
			"offset": 0,
			"limit": 2,
			"order_bys": "!title"
		}))?;
		println!("{:?}", list_options);
		// list_options.order_by();
		Ok(())
	}
}

// endregion: --- TestBmc
