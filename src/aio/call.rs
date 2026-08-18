//! The async transport trait and its typed extension.

use std::future::Future;
use std::pin::Pin;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::Result;

/// An async JSON-RPC transport.
///
/// Boxing the returned future keeps this trait object-safe, so `&dyn
/// RpcCallAsync` works and every typed method trait is available on it. The
/// `Sync` supertrait is what makes the futures `Send`, and therefore spawnable.
pub trait RpcCallAsync: Sync {
    /// Perform one JSON-RPC call.
    ///
    /// `params` is a JSON array for positional arguments or a JSON object for
    /// named arguments; Bitcoin Core accepts either.
    fn call_raw<'a>(
        &'a self,
        method: &'a str,
        params: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>>;
}

/// Typed calling convenience, blanket-implemented for every [`RpcCallAsync`].
///
/// This is the hook for adding your own RPC methods: declare a trait bounded on
/// [`RpcCallAsync`], give its methods default bodies that call
/// [`RpcCallAsyncExt::call`], and blanket-implement it.
pub trait RpcCallAsyncExt: RpcCallAsync {
    /// Call `method` and deserialize the result into `R`.
    fn call<'a, R: DeserializeOwned>(
        &'a self,
        method: &'a str,
        params: Value,
    ) -> impl Future<Output = Result<R>> + Send + 'a {
        async move {
            let value = self.call_raw(method, params).await?;
            Ok(serde_json::from_value(value)?)
        }
    }
}

impl<T: RpcCallAsync + ?Sized> RpcCallAsyncExt for T {}
