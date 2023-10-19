use crate::{
	model::user::UserBmc,
	model::ModelManager,
	pwd::{self, ContentToHash},
	web::{Error, Result},
};
use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::debug;

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/api/login", post(api_login_handler))
		.with_state(mm)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
	username: String,
	password: String,
}

async fn api_login_handler(
	State(mm): State<ModelManager>,
	Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_login_handler", "HANDLER");

	let LoginPayload { username, password } = payload;

	let user = UserBmc::first_by_username(&mm, &username)
		.await?
		.ok_or(Error::LoginFailUsernameNotFound)?;
	let user_id = user.id;
	if user.password == password {
		// Create the success body.
		let body = Json(json!({
			"result": {
				"success": true
			}
		}));
		return Ok(body);
	}
	return Err(Error::LoginFailPwdNotMatching { user_id });
}
