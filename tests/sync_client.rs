#![cfg(feature = "sync")]

mod common;

use bitcoin_rpc::sync::{
    BlockchainRpc, ClientBuilder, MempoolRpc, MiningRpc, NetworkRpc, RpcCall, RpcCallExt,
};
use bitcoin_rpc::types::BlockTemplateRequest;
use bitcoin_rpc::{Auth, Error};
use common::fixtures::{
    BLOCK_TEMPLATE_REPLY, BLOCK_WITH_TXS_REPLY, MEMPOOL_ENTRY_REPLY, PEER_INFO_REPLY,
};
use serde_json::json;

#[test]
fn successful_call_returns_result_and_sends_jsonrpc_2_0() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"blocks":42}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let v = client.call_raw("getblockchaininfo", json!([])).unwrap();
    assert_eq!(v, json!({"blocks": 42}));

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["jsonrpc"], "2.0");
    assert_eq!(sent["method"], "getblockchaininfo");
    assert_eq!(sent["params"], json!([]));
    assert!(sent["id"].is_number());
}

#[test]
fn typed_call_deserializes() {
    #[derive(serde::Deserialize)]
    struct Info {
        blocks: u64,
    }

    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"blocks":42}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let info: Info = client.call("getblockchaininfo", json!([])).unwrap();
    assert_eq!(info.blocks, 42);
}

#[test]
fn rpc_error_becomes_error_rpc() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-8,"message":"Block not found"}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    match client.call_raw("getblockhash", json!([999999999])) {
        Err(Error::Rpc(e)) => {
            assert_eq!(e.code, -8);
            assert_eq!(e.message, "Block not found");
        }
        other => panic!("expected Rpc error, got {other:?}"),
    }
}

#[test]
fn basic_auth_header_is_sent() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":true}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url())
        .auth(Auth::user_pass("user", "pass"))
        .build()
        .unwrap();

    client.call_raw("uptime", json!([])).unwrap();
    assert_eq!(
        server.requests()[0].authorization.as_deref(),
        Some("Basic dXNlcjpwYXNz")
    );
}

#[test]
fn unreachable_node_is_a_transport_error() {
    // Port 1 on loopback refuses connections.
    let client = ClientBuilder::new("http://127.0.0.1:1").build().unwrap();
    assert!(matches!(
        client.call_raw("uptime", json!([])),
        Err(Error::Transport(_))
    ));
}

#[test]
fn bad_url_fails_at_build_time() {
    assert!(matches!(
        ClientBuilder::new("127.0.0.1:8332").build(),
        Err(Error::Config(_))
    ));
}

#[test]
fn http_error_status_with_error_body_is_reported_as_rpc_error() {
    // A 1.0-style node or a proxy may answer 500; the body still carries the error.
    let server = common::MockServer::spawn(vec![(
        500,
        r#"{"result":null,"error":{"code":-32601,"message":"Method not found"},"id":1}"#
            .to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    match client.call_raw("nosuchmethod", json!([])) {
        Err(Error::Rpc(e)) => assert_eq!(e.code, -32601),
        other => panic!("expected Rpc error, got {other:?}"),
    }
}

#[test]
fn default_auth_sends_no_authorization_header() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":true}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    client.call_raw("uptime", json!([])).unwrap();
    assert_eq!(server.requests()[0].authorization, None);
}

#[test]
fn malformed_response_body_is_a_json_error() {
    let server = common::MockServer::spawn(vec![(200, "not json at all".to_string())]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    assert!(matches!(
        client.call_raw("uptime", json!([])),
        Err(Error::Json(_))
    ));
}

#[test]
fn get_block_with_txs_sends_verbosity_2_and_deserializes() {
    let server = common::MockServer::spawn(vec![(200, BLOCK_WITH_TXS_REPLY.to_string())]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let block = client
        .get_block_with_txs("00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09")
        .unwrap();
    assert_eq!(block.height, 100);
    assert_eq!(block.stripped_size, 285);
    assert_eq!(block.coinbase_tx.sequence, 4294967295);
    assert_eq!(block.coinbase_tx.witness.as_deref(), Some("00"));
    assert_eq!(block.tx.len(), 1);
    assert_eq!(block.tx[0].fee, Some(0.00012345));
    assert!(block.previous_block_hash.is_some());
    assert_eq!(block.next_block_hash, None);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getblock");
    assert_eq!(
        sent["params"],
        json!([
            "00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09",
            2
        ])
    );
}

#[test]
fn get_tx_out_miss_returns_none_and_omits_default_include_mempool() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":null}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    assert_eq!(client.get_tx_out("abc123", 0, None).unwrap(), None);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "gettxout");
    assert_eq!(sent["params"], json!(["abc123", 0]));
}

#[test]
fn get_deployment_info_omits_absent_block_hash() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"hash":"0f91","height":0,"script_flags":["P2SH"],"deployments":{"segwit":{"type":"buried","height":0,"active":true}}}}"#
            .to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let info = client.get_deployment_info(None).unwrap();
    assert_eq!(info.script_flags, vec!["P2SH"]);
    assert_eq!(info.deployments["segwit"].deployment_type, "buried");
    assert_eq!(info.deployments["segwit"].bip9, None);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["params"], json!([]));
}

#[test]
fn wait_for_new_block_keeps_interior_null_timeout() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"hash":"0f91","height":7}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let tip = client.wait_for_new_block(None, Some("dead")).unwrap();
    assert_eq!(tip.height, 7);

    // An omitted `timeout` before a present `current_tip` must stay an explicit
    // null so the node still sees `current_tip` in position 1.
    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "waitfornewblock");
    assert_eq!(sent["params"], json!([null, "dead"]));
}

#[test]
fn get_mempool_entry_sends_txid_and_deserializes() {
    let server = common::MockServer::spawn(vec![(200, MEMPOOL_ENTRY_REPLY.to_string())]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let entry = client
        .get_mempool_entry("2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866")
        .unwrap();
    assert_eq!(entry.vsize, 204);
    assert_eq!(entry.height, 800000);
    assert_eq!(entry.fees.base, 0.00012345);
    assert_eq!(entry.depends.len(), 1);
    assert!(entry.spent_by.is_empty());
    assert!(!entry.unbroadcast);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getmempoolentry");
    assert_eq!(
        sent["params"],
        json!(["2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866"])
    );
}

#[test]
fn get_raw_mempool_with_sequence_sends_verbose_false_and_sequence_true() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"txids":["abc123"],"mempool_sequence":42}}"#
            .to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let seq = client.get_raw_mempool_with_sequence().unwrap();
    assert_eq!(seq.txids, vec!["abc123"]);
    assert_eq!(seq.mempool_sequence, 42);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getrawmempool");
    assert_eq!(sent["params"], json!([false, true]));
}

#[test]
fn get_peer_info_sends_no_params_and_deserializes() {
    let server = common::MockServer::spawn(vec![(200, PEER_INFO_REPLY.to_string())]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let peers = client.get_peer_info().unwrap();
    assert_eq!(peers.len(), 1);
    let peer = &peers[0];
    assert_eq!(peer.id, 7);
    assert_eq!(peer.addr_bind.as_deref(), Some("10.0.0.1:8333"));
    // A peer's clock can lag ours, so timeoffset is signed.
    assert_eq!(peer.time_offset, -2);
    assert_eq!(peer.presynced_headers, -1);
    assert_eq!(peer.bytes_sent_per_msg.get("ping"), Some(&32));
    assert_eq!(peer.bytes_recv_per_msg.get("pong"), Some(&32));
    assert_eq!(peer.connection_type, "outbound-full-relay");

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getpeerinfo");
    assert_eq!(sent["params"], json!([]));
}

#[test]
fn disconnect_node_by_id_only_sends_leading_null_address() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":null}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    client.disconnect_node(None, Some(7)).unwrap();

    // `address` is omitted but not trailing (nodeid follows), so it must stay
    // an explicit null rather than being dropped.
    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "disconnectnode");
    assert_eq!(sent["params"], json!([null, 7]));
}

#[test]
fn add_node_omits_absent_v2transport() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":null}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    client.add_node("192.168.0.6:8333", "onetry", None).unwrap();

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "addnode");
    assert_eq!(sent["params"], json!(["192.168.0.6:8333", "onetry"]));
}

#[test]
fn get_block_template_sends_request_object_and_deserializes() {
    let server = common::MockServer::spawn(vec![(200, BLOCK_TEMPLATE_REPLY.to_string())]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let request = BlockTemplateRequest::default();
    let template = client.get_block_template(&request).unwrap();
    assert_eq!(template.height, 800001);
    assert_eq!(template.coinbase_value, 625000000);
    assert_eq!(template.transactions.len(), 1);
    assert_eq!(template.transactions[0].fee, 1000);
    assert_eq!(template.vb_available.get("!testdummy"), Some(&28));
    assert_eq!(template.weight_limit, None);

    // Sent as a single object argument, not a bare array, and the default
    // request carries the segwit rule Core requires.
    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getblocktemplate");
    assert_eq!(sent["params"], json!([{"rules": ["segwit"]}]));
}

#[test]
fn submit_block_returns_none_on_success() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":null}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    assert_eq!(client.submit_block("aabbcc").unwrap(), None);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "submitblock");
    assert_eq!(sent["params"], json!(["aabbcc"]));
}

#[test]
fn submit_block_returns_rejection_reason() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":"duplicate"}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    assert_eq!(
        client.submit_block("aabbcc").unwrap(),
        Some("duplicate".to_string())
    );
}

#[test]
fn get_network_hash_ps_sends_both_args() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":1234.5}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let hps = client.get_network_hash_ps(Some(120), Some(-1)).unwrap();
    assert_eq!(hps, 1234.5);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getnetworkhashps");
    assert_eq!(sent["params"], json!([120, -1]));
}
