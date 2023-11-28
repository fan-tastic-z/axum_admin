use crate::web::{self, remove_token_cookie, Error, Result};
use axum::{extract::State, routing::post, Json, Router};
use lib_auth::pwd::{self, ContentToHash, SchemeStatus};
use lib_core::{
	ctx::Ctx,
	model::{
		user::{UserBmc, UserForLogin},
		ModelManager,
	},
};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::Cookies;
use tracing::debug;

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/api/login", post(api_login_handler))
		.route("/api/logoff", post(api_logoff_handler))
		.with_state(mm)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
	username: String,
	password: String,
}

async fn api_login_handler(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_login_handler", "HANDLER");

	let LoginPayload {
		username,
		password: pwd_clear,
	} = payload;
	let root_ctx = Ctx::root_ctx();

	let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
		.await?
		.ok_or(Error::LoginFailUsernameNotFound)?
		.into();

	let user_id = user.id;

	let Some(pwd) = user.password else {
		return Err(Error::LoginFailUserHasNoPwd { user_id });
	};

	let scheme_status = pwd::validate_pwd(
		&ContentToHash {
			salt: user.password_salt,
			content: pwd_clear.clone(),
		},
		&pwd,
	)
	.map_err(|cause| Error::LoginFail { user_id, cause })?;

	// -- Update password scheme if need
	if let SchemeStatus::Outdated = scheme_status {
		debug!("pwd encrypt scheme outdated, upgrading.");
		UserBmc::update_pwd(&root_ctx, &mm, user.id, &pwd_clear).await?;
	}

	// -- Set web token.
	web::set_token_cookie(&cookies, &user.username, user.token_salt)?;

	let body = Json(json!({
		"result": {
			"success": true
		}
	}));
	Ok(body)
}

// region:    --- Logoff
async fn api_logoff_handler(
	cookies: Cookies,
	Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_logoff_handler", "HANDLER");
	let should_logoff = payload.logoff;

	if should_logoff {
		remove_token_cookie(&cookies)?;
	}

	// Create the success body.
	let body = Json(json!({
		"result": {
			"logged_off": should_logoff
		}
	}));

	Ok(body)
}

#[derive(Debug, Deserialize)]
struct LogoffPayload {
	logoff: bool,
}
// endregion: --- Logoff
