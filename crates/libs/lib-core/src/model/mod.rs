pub mod entity;
mod error;
pub mod modql_utils;
pub mod project;
mod store;
pub mod task;
pub mod user;
use modql::filter::ListOptions;

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
		let list_options: ListOptions = serde_json::from_value(json! ({
			"offset": 0,
			"limit": 2,
			"order_bys": "!title"
		}))?;
		if let Some(order_bys) = list_options.order_bys {
			for order_by in order_bys.into_iter() {
				match order_by {
					modql::filter::OrderBy::Asc(col) => {}
					modql::filter::OrderBy::Desc(col) => todo!(),
				}
			}
			// order_bys.into_iter().map(|i| match i {
			// 	modql::filter::OrderBy::Asc(col) => {

			// 	},
			// 	modql::filter::OrderBy::Desc(col) => todo!(),
			// });
			// for i in order_bys.into_iter() {
			// 	match i {
			// 		modql::filter::OrderBy::Asc(col) => todo!(),
			// 		modql::filter::OrderBy::Desc(col) => todo!(),
			// 	}
			// }
			// for (col, order) in order_bys.into_sea_col_order_iter() {
			// 	println!("{:?} {:?}", col, order);
			// }
		}
		// list_options.order_by();
		Ok(())
	}
}

// endregion: --- TestBmc
