#![allow(unused)] // For beginning only.

use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
	let hc = httpc_test::new_client("http://localhost:20000")?;

	let req_login = hc.do_post(
		"/api/login",
		json!({
			"username": "admin2",
			"password": "admin"
		}),
	);
	req_login.await?.print().await?;
	Ok(())
}
