use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;
use tracing::debug;
use uuid::Uuid;

use crate::{model, web};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
pub enum Error {
	// -- Login
	LoginFailUsernameNotFound,
	LoginFailUserHasNoPwd { user_id: Uuid },
	LoginFailPwdNotMatching { user_id: Uuid },

	// -- CtxExtError
	CtxExt(web::mw_res_map::CtxExtError),

	// -- Modules
	Model(model::Error),
}

// region:    --- Froms
impl From<model::Error> for Error {
	fn from(val: model::Error) -> Self {
		Error::Model(val)
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
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
			LoginFailUsernameNotFound | LoginFailPwdNotMatching { .. } => {
				(StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
			}
			// -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR,
			),
		}
	}
}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	SERVICE_ERROR,
}
