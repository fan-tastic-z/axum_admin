use axum::{
	extract::State,
	response::{IntoResponse, Response},
	routing::post,
	Json, Router,
};
use lib_core::model::ModelManager;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

mod params;
mod project_rpc;
mod router;
mod state;
mod task_rpc;
use params::*;
pub use state::*;

use crate::web::mw_auth::CtxW;

use crate::web::rpc::router::RpcRouter;

/// The raw JSON-RPC request object, serving as the foundation for RPC routing.
#[derive(Deserialize)]
struct RpcRequest {
	id: Option<Value>,
	method: String,
	params: Option<Value>,
}

/// RPC basic information containing the rpc request
/// id and method for additional logging purposes.
#[derive(Debug)]
pub struct RpcInfo {
	pub id: Option<Value>,
	pub method: String,
}


#[derive(Clone)]
struct RpcStates(ModelManager, Arc<RpcRouter>);

pub fn routes(rpc_state: RpcState) -> Router {
	// Build the combined RpcRouter.
	let rpc_router = RpcRouter::new()
		.extend(task_rpc::rpc_router())
		.extend(project_rpc::rpc_router());

	// Build the Axum Router for '/rpc'
	Router::new()
		.route("/rpc", post(rpc_axum_handler))
		.with_state((rpc_state, Arc::new(rpc_router)))
}

async fn rpc_axum_handler(
	State((rpc_state, rpc_router)): State<(RpcState, Arc<RpcRouter>)>,
	ctx: CtxW,
	Json(rpc_req): Json<RpcRequest>,
) -> Response {
	let ctx = ctx.0;

	// -- Create the RPC Info
	//    (will be set to the response.extensions)
	let rpc_info = RpcInfo {
		id: rpc_req.id.clone(),
		method: rpc_req.method.clone(),
	};

	// -- Exec Rpc Route
	let res = rpc_router
		.call(&rpc_info.method, ctx, rpc_state, rpc_req.params)
		.await;

	// -- Build Rpc Success Response
	let res = res.map(|v| {
		let body_response = json!({
			"id": rpc_info.id,
			"result": v
		});
		Json(body_response)
	});

	// -- Create and Update Axum Response
	let mut res = res.into_response();
	res.extensions_mut().insert(rpc_info);

	res
}
