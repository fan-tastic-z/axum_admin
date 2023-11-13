use crate::web::{
	rpc::task_rpc::{create_task, delete_task, list_tasks, update_task},
	Error, Result,
};
use axum::{
	extract::State,
	response::{IntoResponse, Response},
	routing::post,
	Json, Router,
};
use lib_core::{ctx::Ctx, model::ModelManager};
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use tracing::debug;

mod params;
mod task_rpc;
use params::*;

use crate::web::mw_auth::CtxW;

/// The raw JSON-RPC request object, serving as the foundation for RPC routing.
#[derive(Deserialize)]
struct RpcRequest {
	id: Option<Value>,
	method: String,
	params: Option<Value>,
}

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/rpc", post(rpc_handler))
		.with_state(mm)
}

async fn rpc_handler(
	State(mm): State<ModelManager>,
	ctx: CtxW,
	Json(rpc_req): Json<RpcRequest>,
) -> Response {
	let ctx = ctx.0;
	// -- Create the RPC Info to be set to the response.extensions.
	let rpc_info = RpcInfo {
		id: rpc_req.id.clone(),
		method: rpc_req.method.clone(),
	};

	// -- Exec & Store RpcInfo in response.
	let mut res = _rpc_handler(ctx, mm, rpc_req).await.into_response();
	res.extensions_mut().insert(rpc_info);

	res
}

/// RPC basic information containing the rpc request
/// id and method for additional logging purposes.
#[derive(Debug)]
pub struct RpcInfo {
	pub id: Option<Value>,
	pub method: String,
}

macro_rules! exec_rpc_fn {
	// With optional Params
	($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr, "optional_params") => {{
		let rpc_fn_name = stringify!($rpc_fn);

		let params = $rpc_params.map(from_value).transpose().map_err(|ex| {
			Error::RpcFailJsonParams {
				rpc_method: rpc_fn_name.to_string(),
				cause: ex.to_string(),
			}
		})?;

		$rpc_fn($ctx, $mm, params).await.map(to_value)??
	}};

	// With Params
	($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{
		let rpc_fn_name = stringify!($rpc_fn);
		let params = $rpc_params.ok_or(Error::RpcMissingParams {
			rpc_method: rpc_fn_name.to_string(),
		})?;
		let params = from_value(params).map_err(|ex| Error::RpcFailJsonParams {
			rpc_method: rpc_fn_name.to_string(),
			cause: ex.to_string(),
		})?;
		$rpc_fn($ctx, $mm, params).await.map(to_value)??
	}};

	// Without Params
	($rpc_fn:expr, $ctx:expr, $mm:expr) => {
		$rpc_fn($ctx, $mm).await.map(to_value)??
	};
}

async fn _rpc_handler(
	ctx: Ctx,
	mm: ModelManager,
	rpc_req: RpcRequest,
) -> Result<Json<Value>> {
	let RpcRequest {
		id: rpc_id,
		method: rpc_method,
		params: rpc_params,
	} = rpc_req;

	debug!("{:<12} - _rpc_handler - method: {rpc_method}", "HANDLER");

	let result_json: Value = match rpc_method.as_str() {
		// -- Task RPC methods.
		"create_task" => exec_rpc_fn!(create_task, ctx, mm, rpc_params),
		"list_tasks" => {
			exec_rpc_fn!(list_tasks, ctx, mm, rpc_params, "optional_params")
		}
		"update_task" => exec_rpc_fn!(update_task, ctx, mm, rpc_params),
		"delete_task" => exec_rpc_fn!(delete_task, ctx, mm, rpc_params),

		// -- Fallback as Err.
		_ => return Err(Error::RpcMethodUnknown(rpc_method)),
	};

	let body_response = json!({
		"id": rpc_id,
		"result": result_json
	});

	Ok(Json(body_response))
}
