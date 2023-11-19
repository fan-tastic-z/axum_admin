use axum::{http::StatusCode, response::IntoResponse};
use lib_core::{model, pwd, token};
use serde::Serialize;
use tracing::debug;
use uuid::Uuid;

use crate::web;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
	// -- RPC
	RpcMethodUnknown(String),
	RpcMissingParams { rpc_method: String },
	RpcFailJsonParams { rpc_method: String, cause: String },

	// -- Login
	LoginFailUsernameNotFound,
	LoginFailUserHasNoPwd { user_id: Uuid },
	LoginFail { user_id: Uuid, cause: pwd::Error },

	// -- ReqStamp
	ReqStampNotInResponseExt,

	// -- CtxExtError
	CtxExt(web::mw_auth::CtxExtError),

	// -- Modules
	Model(model::Error),
	Pwd(pwd::Error),
	Token(token::Error),

	// -- External Modules
	SerdeJson(String),
}

// region:    --- Froms
impl From<model::Error> for Error {
	fn from(val: model::Error) -> Self {
		Error::Model(val)
	}
}

impl From<pwd::Error> for Error {
	fn from(val: pwd::Error) -> Self {
		Self::Pwd(val)
	}
}

impl From<token::Error> for Error {
	fn from(val: token::Error) -> Self {
		Self::Token(val)
	}
}

impl From<serde_json::Error> for Error {
	fn from(val: serde_json::Error) -> Self {
		Self::SerdeJson(val.to_string())
	}
}

// endregion: --- Froms

// region:    --- Axum IntoResponse

impl IntoResponse for Error {
	fn into_response(self) -> axum::response::Response {
		debug!("{:<12} - model::Error {self:?}", "INTO_RES");
		// Create a placeholder Axum reponse.
		let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
		// Insert the Error into the reponse.
		response.extensions_mut().insert(self);
		response
	}
}

// endregion: --- Axum IntoResponse

// region:    --- Error Biolerplate

impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

// endregion: --- Error Biolerplate

impl Error {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		use web::Error::*;

		#[allow(unreachable_patterns)]
		match self {
			// -- Login
			LoginFailUsernameNotFound
			| LoginFailUserHasNoPwd { .. }
			| LoginFail { .. } => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

			// -- Auth
			CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

			// -- Model
			Model(model::Error::EntityNotFound { entity, id }) => (
				StatusCode::BAD_REQUEST,
				ClientError::ENTITY_NOT_FOUND { entity, id: *id },
			),

			// -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR,
			),
		}
	}
}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
	ENTITY_NOT_FOUND { entity: &'static str, id: Uuid },

	SERVICE_ERROR,
}
