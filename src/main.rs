use std::net::SocketAddr;

use axum::Router;
use tracing_subscriber::EnvFilter;
use web::routes_static;

mod error;
mod web;
pub use self::error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.without_time()
		.with_target(false)
		.with_env_filter(EnvFilter::from_default_env())
		.init();

	let routers_all = Router::new().fallback_service(routes_static::serve_dir());

	// region:    --- Start Server
	let addr = SocketAddr::from(([127, 0, 0, 1], 20000));
	axum::Server::bind(&addr)
		.serve(routers_all.into_make_service())
		.await
		.unwrap();
	// endregion: --- Start Server
	Ok(())
}
