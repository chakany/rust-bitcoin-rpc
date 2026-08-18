#![cfg(feature = "aio")]

mod common;

use bitcoin_rpc::aio::{ClientBuilder, RpcCallAsync, RpcCallAsyncExt};
use bitcoin_rpc::{Auth, Error};
use serde_json::json;

#[tokio::test]
async fn successful_call_returns_result() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"blocks":42}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let v = client.call_raw("getblockchaininfo", json!([])).await.unwrap();
    assert_eq!(v, json!({"blocks": 42}));

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["jsonrpc"], "2.0");
    assert_eq!(sent["method"], "getblockchaininfo");
}

#[tokio::test]
async fn typed_call_deserializes() {
    #[derive(serde::Deserialize)]
    struct Info {
        blocks: u64,
    }

    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"blocks":42}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let info: Info = client.call("getblockchaininfo", json!([])).await.unwrap();
    assert_eq!(info.blocks, 42);
}

#[tokio::test]
async fn rpc_error_becomes_error_rpc() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-8,"message":"Block not found"}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    match client.call_raw("getblockhash", json!([999999999])).await {
        Err(Error::Rpc(e)) => assert_eq!(e.code, -8),
        other => panic!("expected Rpc error, got {other:?}"),
    }
}

#[tokio::test]
async fn basic_auth_header_is_sent() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":true}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url())
        .auth(Auth::user_pass("user", "pass"))
        .build()
        .unwrap();

    client.call_raw("uptime", json!([])).await.unwrap();
    assert_eq!(
        server.requests()[0].authorization.as_deref(),
        Some("Basic dXNlcjpwYXNz")
    );
}

#[tokio::test]
async fn unreachable_node_is_a_transport_error() {
    let client = ClientBuilder::new("http://127.0.0.1:1").build().unwrap();
    assert!(matches!(
        client.call_raw("uptime", json!([])).await,
        Err(Error::Transport(_))
    ));
}

#[tokio::test]
async fn client_is_shareable_and_futures_are_spawnable() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":123}"#.to_string(),
    )]);
    let client = std::sync::Arc::new(ClientBuilder::new(server.url()).build().unwrap());

    // The `Sync` supertrait exists so this future is Send.
    let handle = tokio::spawn(async move { client.call_raw("uptime", json!([])).await });
    assert_eq!(handle.await.unwrap().unwrap(), json!(123));
}
