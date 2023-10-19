use crate::{
	log::log_request,
	web::{Error, Result},
};
use async_trait::async_trait;
use axum::{
	extract::FromRequestParts,
	http::{request::Parts, Method, Request, Uri},
	middleware::Next,
	response::{IntoResponse, Response},
	Json,
};
use serde::Serialize;
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

use crate::{ctx::Ctx, web};

pub async fn mw_response_map(
	ctx: Option<Ctx>,
	uri: Uri,
	req_method: Method,
	res: Response,
) -> Response {
	debug!("{:<12} - mw_reponse_map", "RES_MAPPER");
	let uuid = Uuid::new_v4();
	// -- Get the eventual response error.
	let web_error = res.extensions().get::<web::Error>();
	let client_status_error = web_error.map(|se| se.client_status_and_error());
	let error_response =
		client_status_error
			.as_ref()
			.map(|(status_code, client_error)| {
				let client_error_body = json!({
					"error": {
						"type": client_error.as_ref(),
					}
				});
				debug!("CLIENT ERROR BODY:\n{client_error_body}");

				// Build the new response from the client_error_body
				(*status_code, Json(client_error_body)).into_response()
			});
	let client_error = client_status_error.unzip().1;
	let _ = log_request(uuid, req_method, uri, ctx, web_error, client_error).await;

	error_response.unwrap_or(res)
}

pub async fn mw_ctx_resolve<B>(req: Request<B>, next: Next<B>) -> Result<Response> {
	Ok(next.run(req).await)
}

// region:    --- Ctx Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
		debug!("{:<12} - Ctx", "EXTRACTOR");
		parts
			.extensions
			.get::<CtxExtResult>()
			.ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
			.clone()
			.map_err(Error::CtxExt)
	}
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<Ctx, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
	CtxNotInRequestExt,
}
// endregion: --- Ctx Extractor Result/Error
