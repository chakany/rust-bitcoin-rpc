// This whole file exercises extending `RpcCall` / `RpcCallAsync`, so with
// neither `sync` nor `aio` enabled there is nothing to test.
#![cfg(any(feature = "sync", feature = "aio"))]

// `common` (including `common::fixtures`) is shared across integration test
// binaries; the other consumers (`sync_client.rs`, `aio_client.rs`) use every
// item in it, but this file only needs `MockServer`, so unused-item lints
// would otherwise fire purely because of what this particular binary doesn't
// happen to reuse from the shared fixture module.
#[allow(dead_code)]
mod common;

use serde_json::json;

/// A response type a user of this crate would define themselves.
///
/// `pub` (not private) because the async extension trait below returns it
/// from a return-position `impl Future` in a `pub` trait: that desugars to a
/// public associated type, so the type it names must be at least as visible
/// as the trait itself — exactly the constraint a real downstream crate
/// would hit and satisfy by making its response type `pub`.
#[derive(Debug, serde::Deserialize)]
pub struct OrphanTx {
    txid: String,
}

#[cfg(feature = "sync")]
mod sync_ext {
    use super::*;
    use bitcoin_rpc::Result;
    use bitcoin_rpc::sync::{RpcCall, RpcCallExt};

    /// Written exactly as a downstream crate would, for an RPC we do not ship.
    pub trait OrphanRpc: RpcCall {
        fn get_orphan_txs(&self) -> Result<Vec<OrphanTx>> {
            self.call("getorphantxs", json!([2]))
        }
    }
    impl<T: RpcCall + ?Sized> OrphanRpc for T {}

    /// It must work through a trait object too.
    pub fn via_dyn(client: &dyn RpcCall) -> Result<usize> {
        Ok(client.get_orphan_txs()?.len())
    }
}

#[cfg(feature = "sync")]
#[test]
fn user_extension_works_on_client_and_on_dyn() {
    use bitcoin_rpc::sync::{ClientBuilder, RpcCall};
    use sync_ext::OrphanRpc;

    let body = r#"{"jsonrpc":"2.0","id":1,"result":[{"txid":"aa"},{"txid":"bb"}]}"#;
    let server = common::MockServer::spawn(vec![(200, body.to_string()), (200, body.to_string())]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    // directly on the concrete client
    let txs = client.get_orphan_txs().unwrap();
    assert_eq!(txs[0].txid, "aa");

    // and through a trait object
    let boxed: Box<dyn RpcCall> = Box::new(ClientBuilder::new(server.url()).build().unwrap());
    assert_eq!(sync_ext::via_dyn(boxed.as_ref()).unwrap(), 2);

    // the params we promised
    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getorphantxs");
    assert_eq!(sent["params"], json!([2]));
}

#[cfg(feature = "sync")]
#[test]
fn user_extension_composes_with_a_shipped_trait_on_the_same_client() {
    use bitcoin_rpc::sync::{ClientBuilder, ControlRpc};
    use sync_ext::OrphanRpc;

    let server = common::MockServer::spawn(vec![
        (
            200,
            r#"{"jsonrpc":"2.0","id":1,"result":[{"txid":"aa"}]}"#.to_string(),
        ),
        (
            200,
            r#"{"jsonrpc":"2.0","id":1,"result":12345}"#.to_string(),
        ),
    ]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    // With the user's `OrphanRpc` and the crate's own `ControlRpc` both in
    // scope, both are callable on the same client value: the extension
    // mechanism does not shadow or otherwise interfere with the crate's
    // shipped traits.
    let orphans = client.get_orphan_txs().unwrap();
    assert_eq!(orphans[0].txid, "aa");
    assert_eq!(client.uptime().unwrap(), 12345);
}

#[cfg(feature = "aio")]
mod aio_ext {
    use super::*;
    use bitcoin_rpc::Result;
    use bitcoin_rpc::aio::{RpcCallAsync, RpcCallAsyncExt};
    use std::future::Future;

    /// Written exactly as a downstream crate would, for an RPC we do not ship.
    pub trait OrphanRpc: RpcCallAsync {
        fn get_orphan_txs(&self) -> impl Future<Output = Result<Vec<OrphanTx>>> + Send + '_ {
            self.call("getorphantxs", json!([2]))
        }
    }
    impl<T: RpcCallAsync + ?Sized> OrphanRpc for T {}

    /// It must work through a trait object too.
    pub async fn via_dyn(client: &dyn RpcCallAsync) -> Result<usize> {
        Ok(client.get_orphan_txs().await?.len())
    }
}

#[cfg(feature = "aio")]
#[tokio::test]
async fn user_async_extension_works_on_client_and_on_dyn() {
    use aio_ext::OrphanRpc;
    use bitcoin_rpc::aio::{ClientBuilder, RpcCallAsync};

    let body = r#"{"jsonrpc":"2.0","id":1,"result":[{"txid":"aa"},{"txid":"bb"}]}"#;
    let server = common::MockServer::spawn(vec![(200, body.to_string()), (200, body.to_string())]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    // directly on the concrete client
    let txs = client.get_orphan_txs().await.unwrap();
    assert_eq!(txs[0].txid, "aa");

    // and through a trait object: pins the claim at src/aio/call.rs that
    // boxing the returned future keeps `RpcCallAsync` object-safe.
    let boxed: Box<dyn RpcCallAsync> = Box::new(ClientBuilder::new(server.url()).build().unwrap());
    assert_eq!(aio_ext::via_dyn(boxed.as_ref()).await.unwrap(), 2);

    // the params we promised
    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getorphantxs");
    assert_eq!(sent["params"], json!([2]));
}

#[cfg(feature = "aio")]
#[tokio::test]
async fn user_async_extension_works_and_is_spawnable() {
    use aio_ext::OrphanRpc;
    use bitcoin_rpc::aio::ClientBuilder;

    let body = r#"{"jsonrpc":"2.0","id":1,"result":[{"txid":"aa"}]}"#;
    let server = common::MockServer::spawn(vec![(200, body.to_string())]);
    let client = std::sync::Arc::new(ClientBuilder::new(server.url()).build().unwrap());

    let handle = tokio::spawn(async move { client.get_orphan_txs().await });
    assert_eq!(handle.await.unwrap().unwrap()[0].txid, "aa");
}

#[cfg(feature = "sync")]
#[test]
fn prelude_brings_the_method_traits_into_scope() {
    // With only `sync` enabled the flat glob carries the method traits; with
    // `aio` also enabled the flat glob is deliberately absent (its contents
    // would collide with the async traits' identical names), so the nested
    // `prelude::sync` module is what a caller reaches for instead.
    #[cfg(feature = "aio")]
    use bitcoin_rpc::prelude::sync::*;
    #[cfg(not(feature = "aio"))]
    use bitcoin_rpc::prelude::*;

    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":12345}"#.to_string(),
    )]);
    let client = bitcoin_rpc::sync::ClientBuilder::new(server.url())
        .build()
        .unwrap();
    // `uptime` resolves only via the prelude import brought into scope above.
    assert_eq!(client.uptime().unwrap(), 12345);
}
