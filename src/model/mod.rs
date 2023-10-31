mod entity;
mod error;
mod store;
pub mod task;
pub mod user;
pub use self::error::{Error, Result};
use self::store::{new_db_pool, Db};

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
