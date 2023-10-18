use axum::{
	response::{IntoResponse, Response},
	Json,
};
use serde_json::json;
use tracing::debug;

use crate::web;

pub async fn mw_response_map(res: Response) -> Response {
	debug!("{:<12} - mw_reponse_map", "RES_MAPPER");
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
	let _ = client_status_error.unzip().1;
	error_response.unwrap_or(res)
}
