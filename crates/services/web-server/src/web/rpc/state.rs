use lib_core::model::ModelManager;

// region:    --- RpcState
/// The RpcState for the RPC handler functions.
///
/// This becomes useful as the application grows and requires states other than
/// the ModelManager in the RpcHandlers.
///
/// By default, any RPC handler can have `my_rpc_handler(Ctx, RpcState, ...)`.
///
/// Implements `From<RpcState>` to allow extracting a sub-state
/// (e.g., `my_rpc_handler(Ctx, ModelManager, ...)`.
#[derive(Clone)]
pub struct RpcState {
	pub mm: ModelManager,
}

/// `RpcState -> ModelManager` allowing rpc handler functions
/// To just have `my_rpc_handler(ctx: Ctx, mm: ModelManager, ..)`
impl From<RpcState> for ModelManager {
	fn from(val: RpcState) -> Self {
		val.mm
	}
}
// endregion: --- RpcState
