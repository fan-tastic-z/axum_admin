pub mod entity;
mod error;
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
	pub limit: i64,
	pub offset: i64,
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
	// pub fn order_by<E>(self, select_query: &mut sea_orm::prelude::Select<E>)
	// where
	// 	E: EntityTrait,
	// {
	// 	if let Some(order_bys) = self.order_bys {
	// 		println!("{:?}", order_bys);
	// 		for (col, order) in order_bys.into_sea_col_order_iter() {
	// 			println!(col)
	// 			// select_query.order_by(col, order);
	// 		}
	// 	}
	// }
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
