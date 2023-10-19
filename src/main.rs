use std::net::SocketAddr;

use axum::{middleware, Router};
use tracing::info;
use tracing_subscriber::EnvFilter;
use web::routes_static;

mod config;
mod ctx;
mod error;
mod model;
mod pwd;
mod token;
mod utils;
mod web;
mod log;
use crate::web::mw_res_map::mw_ctx_resolve;
use crate::web::routes_login;
use crate::{model::ModelManager, web::mw_res_map::mw_response_map};
pub use config::config;

pub use self::error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.without_time()
		.with_target(false)
		.with_env_filter(EnvFilter::from_default_env())
		.init();

	// Initialze ModelManager.
	let mm = ModelManager::new().await?;

	let routers_all = Router::new()
		.merge(routes_login::routes(mm.clone()))
		.layer(middleware::map_response(mw_response_map))
		.layer(middleware::from_fn(mw_ctx_resolve))
		.fallback_service(routes_static::serve_dir());

	// region:    --- Start Server
	let addr = SocketAddr::from(([127, 0, 0, 1], 20000));
	info!("{:<12} - {addr}\n", "LISTENING");
	axum::Server::bind(&addr)
		.serve(routers_all.into_make_service())
		.await
		.unwrap();
	// endregion: --- Start Server
	Ok(())
}
