use crate::router::into_params::IntoParams;
use crate::router::FromResources;
use crate::router::PinFutureValue;
use crate::router::Result;
use crate::router::RpcHandlerWrapper;
use crate::RpcResources;
use futures::Future;
use serde::Serialize;
use serde_json::Value;

/// The `Handler` trait that will be implemented by rpc handler functions.
///
/// Key points:
/// - Rpc handler functions are asynchronous, thus returning a Future of Result<Value>.
/// - The call format is normalized to two `impl FromResources` arguments (for now) and one optionals  `impl IntoParams`, which represent the json-rpc's optional value.
/// - `into_box` is a convenient method for converting a RpcHandler into a Boxed dyn RpcHandlerWrapperTrait,
///   allowing for dynamic dispatch by the Router.
/// - A `RpcHandler` will typically be implemented for static functions, as `FnOnce`,
///   enabling them to be cloned with none or negligible performance impact,
///   thus facilitating the use of RpcRoute dynamic dispatch.
pub trait RpcHandler<S, P, R>: Clone {
	/// The type of future calling this handler returns.
	type Future: Future<Output = Result<Value>> + Send + 'static;

	/// Call the handler.
	fn call(
		self,
		rpc_resources: RpcResources,
		params: Option<Value>,
	) -> Self::Future;

	/// Handler into a Boxed RpcHandlerWrapper
	/// which can then be placed in a container of `Box<dyn RpcHandlerWrapperTrait>`
	/// for dynamic dispatch.
	fn into_box(self) -> Box<RpcHandlerWrapper<Self, S, P, R>> {
		Box::new(RpcHandlerWrapper::new(self))
	}
}

/// RpcHanlder implementation for `my_rpc_handler(FromResources, FromResources) -> Result<Serialize> `
/// (so, without any `IntoParams` last argument).
impl<F, Fut, S1, S2, R> RpcHandler<(S1, S2), (), R> for F
where
	F: FnOnce(S1, S2) -> Fut + Clone + Send + 'static,
	S1: FromResources + Send,
	S2: FromResources + Send,
	R: Serialize,
	Fut: Future<Output = Result<R>> + Send,
{
	type Future = PinFutureValue;

	fn call(
		self,
		rpc_resources: RpcResources,
		_params: Option<Value>,
	) -> Self::Future {
		Box::pin(async move {
			let result = self(
				S1::from_resources(&rpc_resources)?,
				S2::from_resources(&rpc_resources)?,
			)
			.await?;
			Ok(serde_json::to_value(result)?)
		})
	}
}

/// RpcHandler implementation for `my_rpc_handler(FromResources, FromResources, IntoParams) -> Result<Serialize>`.
/// Note: The trait bounds `Clone + Send + 'static` apply to `F`,
///       and `Fut` has its own trait bounds defined afterwards.
impl<F, Fut, S1, S2, P, R> RpcHandler<(S1, S2), (P,), R> for F
where
	F: FnOnce(S1, S2, P) -> Fut + Clone + Send + 'static,
	S1: FromResources + Send,
	S2: FromResources + Send,
	P: IntoParams,
	R: Serialize,
	Fut: Future<Output = Result<R>> + Send,
{
	type Future = PinFutureValue;

	fn call(
		self,
		rpc_resources: RpcResources,
		params_value: Option<Value>,
	) -> Self::Future {
		Box::pin(async move {
			let param = P::into_params(params_value)?;

			let result = self(
				S1::from_resources(&rpc_resources)?,
				S2::from_resources(&rpc_resources)?,
				param,
			)
			.await?;
			Ok(serde_json::to_value(result)?)
		})
	}
}
