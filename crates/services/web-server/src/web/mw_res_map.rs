use std::sync::Arc;

use crate::{log::log_request, web::routes_rpc::RpcInfo};
use axum::{
	http::{Method, Uri},
	response::{IntoResponse, Response},
	Json,
};

use serde_json::{json, to_value};
use tracing::{debug, error};
use uuid::Uuid;

use crate::web;

use super::mw_auth::CtxW;

pub async fn mw_response_map(
	ctx: Option<CtxW>,
	uri: Uri,
	req_method: Method,
	req_stamp: web::ReqStamp,
	res: Response,
) -> Response {
	let ctx = ctx.map(|c| c.0);

	debug!("{:<12} - mw_reponse_map", "RES_MAPPER");
	let uuid = Uuid::new_v4();
	let rpc_info = res.extensions().get::<Arc<RpcInfo>>();

	// -- Get the eventual response error.
	let web_error = res.extensions().get::<Arc<web::Error>>();
	let client_status_error = web_error.map(|se| se.client_status_and_error());
	let error_response =
		client_status_error
			.as_ref()
			.map(|(status_code, client_error)| {
				let client_error = to_value(client_error).ok();
				let message = client_error.as_ref().and_then(|v| v.get("message"));
				let detail = client_error.as_ref().and_then(|v| v.get("detail"));

				let client_error_body = json!({
					"id": rpc_info.as_ref().map(|rpc| rpc.id.clone()),
					"error": {
						"message": message, // Variant name
						"data": {
							"req_uuid": uuid.to_string(),
							"detail": detail
						},
					}
				});

				debug!("CLIENT ERROR BODY:\n{client_error_body}");

				// Build the new response from the client_error_body
				(*status_code, Json(client_error_body)).into_response()
			});
	let client_error = client_status_error.unzip().1;
	let log_request_res = log_request(
		req_method,
		uri,
		req_stamp,
		rpc_info,
		ctx,
		web_error,
		client_error,
	)
	.await;
	// For now, just simple error trace
	if let Err(err) = log_request_res {
		error!("IMPORTANT - log_request failed - {:?}", err);
	}

	debug!("\n");
	error_response.unwrap_or(res)
}
