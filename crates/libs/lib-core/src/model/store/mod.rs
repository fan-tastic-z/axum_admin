mod error;

use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::config::core_config;

pub use self::error::{Error, Result};

pub type Db = DatabaseConnection;

pub async fn new_db_pool() -> Result<Db> {
	let mut opt = ConnectOptions::new(&core_config().DB_URL);
	opt.max_connections(1000)
		.min_connections(5)
		.connect_timeout(Duration::from_secs(8))
		.idle_timeout(Duration::from_secs(8))
		.sqlx_logging(true);
	Database::connect(opt)
		.await
		.map_err(|ex| Error::FailToCreatePool(ex.to_string()))
}
