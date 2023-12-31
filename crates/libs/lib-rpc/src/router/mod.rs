mod from_resources;
mod into_params;
mod rpc_handler;
mod rpc_handler_wrapper;

use std::{collections::HashMap, pin::Pin};

use crate::{Error, Result, RpcResources};
pub use from_resources::FromResources;
pub use into_params::{IntoDefaultParams, IntoParams};
pub use rpc_handler::RpcHandler;
pub use rpc_handler_wrapper::{RpcHandlerWrapper, RpcHandlerWrapperTrait};

use futures::Future;
use serde::Deserialize;
use serde_json::Value;

/// The raw JSON-RPC request object, serving as the foundation for RPC routing.
#[derive(Deserialize)]
pub struct RpcRequest {
	pub id: Option<Value>,
	pub method: String,
	pub params: Option<Value>,
}

pub type PinFutureValue = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;

/// method, which calls the appropriate handler matching the method_name.
///
/// RpcRouter can be extended with other RpcRouters for composability.
pub struct RpcRouter {
	route_by_name: HashMap<&'static str, Box<dyn RpcHandlerWrapperTrait>>,
}

impl RpcRouter {
	#[allow(clippy::new_without_default)] // Persosnal preference (for this case)
	pub fn new() -> Self {
		Self {
			route_by_name: HashMap::new(),
		}
	}

	/// Add a dyn_handler to the router.
	///
	/// ```
	/// RpcRouter::new().add_dyn("method_name", my_handler_fn.into_dyn());
	/// ```
	///
	/// Note: This is the preferred way to add handlers to the router, as it
	///       avoids monomorphization of the add function.
	///       The RpcRouter also has a `.add()` as a convenience function to just pass the function.
	///       See `RpcRouter::add` for more details.
	pub fn add_dyn(
		mut self,
		name: &'static str,
		dyn_handler: Box<dyn RpcHandlerWrapperTrait>,
	) -> Self {
		self.route_by_name.insert(name, dyn_handler);
		self
	}

	/// Add an handler function to the router.
	///
	/// ```
	/// RpcRouter::new().add("method_name", my_handler_fn);
	/// ```
	///
	/// Note: This is a convenient add function variant with generics,
	///       and there will be monomorphed versions of this function
	///       for each type passed. Use `RpcRouter::add_dyn` to avoid this.
	pub fn add<F, T, P, R>(self, name: &'static str, handler: F) -> Self
	where
		F: RpcHandler<T, P, R> + Clone + Send + Sync + 'static,
		T: Send + Sync + 'static,
		P: Send + Sync + 'static,
		R: Send + Sync + 'static,
	{
		self.add_dyn(name, handler.into_dyn())
	}

	pub fn extend(mut self, other_router: RpcRouter) -> Self {
		self.route_by_name.extend(other_router.route_by_name);
		self
	}

	pub async fn call(
		&self,
		method: &str,
		params: Option<Value>,
		rpc_resources: RpcResources,
	) -> Result<Value> {
		if let Some(route) = self.route_by_name.get(method) {
			route.call(rpc_resources, params).await
		} else {
			Err(Error::RpcMethodUnknown(method.to_string()))
		}
	}
}

/// A simple macro to create a new RpcRouter
/// and add each rpc handler-compatible function along with their corresponding names.
///
/// e.g.,
///
/// ```
/// rpc_router!(
///   create_project,
///   list_projects,
///   update_project,
///   delete_project
/// );
/// ```
/// Is equivalent to:
/// ```
/// RpcRouter::new()
///     .add_dyn("create_project", create_project.into_box())
///     .add_dyn("list_projects", list_projects.into_box())
///     .add_dyn("update_project", update_project.into_box())
///     .add_dyn("delete_project", delete_project.into_box())
/// ```
///
#[macro_export]
macro_rules! rpc_router {
    ($($fn_name:ident),+ $(,)?) => {
        {
			use $crate::router::{RpcHandler, RpcRouter};
            let mut router = RpcRouter::new();
            $(
                router = router.add_dyn(stringify!($fn_name), $fn_name.into_dyn());
            )+
            router
        }
    };
}
