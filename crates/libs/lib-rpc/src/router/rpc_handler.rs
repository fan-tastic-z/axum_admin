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

/// Macro generatring the RpcHandler implementations for zero or more FromResources with the last argument being IntoParams
/// and one with not last IntoParams argument.
macro_rules! impl_rpc_handler {
	($($S:ident),*) => {
		// RpcHandler implementations for zero or more FromResources with the last argument being IntoParams
		impl<F, Fut, $($S,)* P, R> RpcHandler<($($S,)*), (P,), R> for F
        where
            F: FnOnce($($S,)* P) -> Fut + Clone + Send + 'static,
            $( $S: FromResources + Send, )*
            P: IntoParams,
            R: Serialize,
            Fut: Future<Output = Result<R>> + Send,
        {
            type Future = PinFutureValue;

						#[allow(unused)] // somehow rpc_resources will be marked as unused
            fn call(
                self,
                rpc_resources: RpcResources,
                params_value: Option<Value>,
            ) -> Self::Future {
                Box::pin(async move {
                    let param = P::into_params(params_value)?;

                    let result = self(
                        $( $S::from_resources(&rpc_resources)?, )*
                        param,
                    )
                    .await?;
                    Ok(serde_json::to_value(result)?)
                })
            }
        }
		// RpcHandler implementations for zero or more FromResources and NO IntoParams
		impl<F, Fut, $($S,)* R> RpcHandler<($($S,)*), (), R> for F
		where
			F: FnOnce($($S,)*) -> Fut + Clone + Send + 'static,
			$( $S: FromResources + Send, )*
			R: Serialize,
			Fut: Future<Output = Result<R>> + Send,
		{
			type Future = PinFutureValue;

			#[allow(unused)] // somehow rpc_resources will be marked as unused
			fn call(
					self,
					rpc_resources: RpcResources,
					_params: Option<Value>,
			) -> Self::Future {
					Box::pin(async move {
							let result = self(
									$( $S::from_resources(&rpc_resources)?, )*
							)
							.await?;
							Ok(serde_json::to_value(result)?)
					})
			}
		}
	};
}

impl_rpc_handler!();
impl_rpc_handler!(S1);
impl_rpc_handler!(S1, S2);
impl_rpc_handler!(S1, S2, S3);
impl_rpc_handler!(S1, S2, S3, S4);
impl_rpc_handler!(S1, S2, S3, S4, S5);
