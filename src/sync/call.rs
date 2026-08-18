//! The blocking transport trait and its typed extension.

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::Result;

/// A blocking JSON-RPC transport.
///
/// This is the only trait a client implements, and it is deliberately
/// object-safe: `&dyn RpcCall` and `Box<dyn RpcCall>` both work, and every typed
/// method trait in this crate is available on them.
pub trait RpcCall {
    /// Perform one JSON-RPC call.
    ///
    /// `params` is a JSON array for positional arguments or a JSON object for
    /// named arguments; Bitcoin Core accepts either.
    fn call_raw(&self, method: &str, params: Value) -> Result<Value>;
}

/// Typed calling convenience, blanket-implemented for every [`RpcCall`].
///
/// This is the hook for adding your own RPC methods: declare a trait bounded on
/// [`RpcCall`], give its methods default bodies that call [`RpcCallExt::call`],
/// and blanket-implement it.
pub trait RpcCallExt: RpcCall {
    /// Call `method` and deserialize the result into `R`.
    fn call<R: DeserializeOwned>(&self, method: &str, params: Value) -> Result<R> {
        Ok(serde_json::from_value(self.call_raw(method, params)?)?)
    }
}

impl<T: RpcCall + ?Sized> RpcCallExt for T {}
