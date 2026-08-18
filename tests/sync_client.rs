#![cfg(feature = "sync")]

mod common;

use bitcoin_rpc::sync::{
    BlockchainRpc, ClientBuilder, ControlRpc, FeeRpc, MempoolRpc, MiningRpc, NetworkRpc,
    RawTransactionsRpc, RpcCall, RpcCallExt, UtilRpc,
};
use bitcoin_rpc::types::{
    BlockTemplateRequest, CreateRawTransactionInput, CreateRawTransactionOutput,
};
use bitcoin_rpc::{Auth, Error};
use common::fixtures::{
    BLOCK_TEMPLATE_REPLY, BLOCK_WITH_TXS_REPLY, MEMPOOL_ENTRY_REPLY, PEER_INFO_REPLY,
    RAW_TRANSACTION_REPLY, TEST_MEMPOOL_ACCEPT_REPLY,
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
    // Reached through the `serde(flatten)`-ed transaction body.
    assert_eq!(block.tx[0].tx.vsize, 204);
    assert_eq!(block.tx[0].tx.vout[0].script_pub_key.script_type, "pubkey");
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

#[test]
fn get_raw_transaction_sends_verbosity_1_and_deserializes() {
    let server = common::MockServer::spawn(vec![(200, RAW_TRANSACTION_REPLY.to_string())]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let tx = client
        .get_raw_transaction(
            "9e1a2d3f4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f6",
            None,
        )
        .unwrap();
    assert_eq!(tx.vsize, 144);
    assert_eq!(tx.confirmations, Some(12));
    assert_eq!(tx.in_active_chain, Some(true));
    assert_eq!(tx.vin[0].vout, Some(1));
    assert_eq!(tx.vin[0].script_sig.as_ref().unwrap().hex, "483045022100");
    assert_eq!(tx.vin[0].tx_in_witness.as_ref().unwrap().len(), 2);
    assert_eq!(tx.vout[0].value, 0.04998);
    assert_eq!(tx.vout[0].script_pub_key.script_type, "witness_v0_keyhash");

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getrawtransaction");
    // The absent `block_hash` is trimmed, but verbosity is still explicit.
    assert_eq!(
        sent["params"],
        json!([
            "9e1a2d3f4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f6",
            1
        ])
    );
}

#[test]
fn create_raw_transaction_sends_both_output_forms() {
    let reply = r#"{"jsonrpc":"2.0","id":1,"result":"0200000001abcdef"}"#.to_string();
    let server = common::MockServer::spawn(vec![(200, reply.clone()), (200, reply)]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let inputs = [CreateRawTransactionInput {
        txid: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b".to_string(),
        vout: 1,
        sequence: None,
    }];
    let outputs = [
        CreateRawTransactionOutput::Address {
            address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
            amount: 0.01,
        },
        CreateRawTransactionOutput::Data("00010203".to_string()),
    ];
    let expected_inputs = json!([{"txid": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b", "vout": 1}]);
    let expected_outputs =
        json!([{"bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4": 0.01}, {"data": "00010203"}]);

    // All five arguments supplied.
    let hex = client
        .create_raw_transaction(&inputs, &outputs, Some(800000), Some(false), Some(3))
        .unwrap();
    assert_eq!(hex, "0200000001abcdef");

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "createrawtransaction");
    assert_eq!(
        sent["params"],
        json!([expected_inputs, expected_outputs, 800000, false, 3])
    );

    // `version` omitted: trimmed as a trailing null, not sent explicitly.
    client
        .create_raw_transaction(&inputs, &outputs, Some(800000), Some(false), None)
        .unwrap();

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[1].body).unwrap();
    assert_eq!(
        sent["params"],
        json!([expected_inputs, expected_outputs, 800000, false])
    );
}

#[test]
fn test_mempool_accept_deserializes_hyphenated_fee_keys() {
    let server = common::MockServer::spawn(vec![(200, TEST_MEMPOOL_ACCEPT_REPLY.to_string())]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let raw_txs = [
        "0200000001abcdef".to_string(),
        "0200000001fedcba".to_string(),
    ];
    let results = client.test_mempool_accept(&raw_txs, None).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].allowed, Some(true));
    let fees = results[0].fees.as_ref().unwrap();
    assert_eq!(fees.base, 0.00001234);
    assert_eq!(fees.effective_feerate, 0.00008567);
    assert_eq!(fees.effective_includes.len(), 1);
    // Validation left unfinished by the first transaction: no `allowed` key.
    assert_eq!(results[1].allowed, None);
    assert_eq!(results[1].fees, None);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "testmempoolaccept");
    assert_eq!(
        sent["params"],
        json!([["0200000001abcdef", "0200000001fedcba"]])
    );
}

#[test]
fn validate_address_sends_positional_address_and_deserializes_the_invalid_shape() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"isvalid":false,"error":"Invalid Bech32 checksum","error_locations":[9,10]}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let addr = client
        .validate_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t5")
        .unwrap();
    assert!(!addr.isvalid);
    assert_eq!(addr.address, None);
    assert_eq!(addr.error.as_deref(), Some("Invalid Bech32 checksum"));
    assert_eq!(addr.error_locations, Some(vec![9, 10]));

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "validateaddress");
    assert_eq!(
        sent["params"],
        json!(["bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t5"])
    );
}

#[test]
fn estimate_smart_fee_sends_both_params_and_deserializes_the_errors_only_shape() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"errors":["Insufficient data or no feerate found"],"blocks":1008}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let est = client.estimate_smart_fee(6, Some("conservative")).unwrap();
    assert_eq!(est.feerate, None);
    assert_eq!(
        est.errors,
        Some(vec!["Insufficient data or no feerate found".to_string()])
    );
    assert_eq!(est.blocks, 1008);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "estimatesmartfee");
    assert_eq!(sent["params"], json!([6, "conservative"]));
}

#[test]
fn get_index_info_deserializes_the_dynamic_map() {
    let server = common::MockServer::spawn(vec![(
        200,
        r#"{"jsonrpc":"2.0","id":1,"result":{"txindex":{"synced":true,"best_block_height":800000}}}"#.to_string(),
    )]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    let map = client.get_index_info(None).unwrap();
    let txindex = map.get("txindex").unwrap();
    assert!(txindex.synced);
    assert_eq!(txindex.best_block_height, 800000);

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[0].body).unwrap();
    assert_eq!(sent["method"], "getindexinfo");
    assert_eq!(sent["params"], json!([]));
}

#[test]
fn control_rpcs_send_expected_params_and_deserialize_string_results() {
    let server = common::MockServer::spawn(vec![
        (200, r#"{"jsonrpc":"2.0","id":1,"result":3600}"#.to_string()),
        (
            200,
            r#"{"jsonrpc":"2.0","id":1,"result":"Bitcoin Core stopping"}"#.to_string(),
        ),
        (
            200,
            r#"{"jsonrpc":"2.0","id":1,"result":"== Blockchain ==\ngetblockcount\n"}"#.to_string(),
        ),
    ]);
    let client = ClientBuilder::new(server.url()).build().unwrap();

    assert_eq!(client.uptime().unwrap(), 3600);
    assert_eq!(client.stop().unwrap(), "Bitcoin Core stopping");
    let help_text = client.help(Some("getblockcount")).unwrap();
    assert!(help_text.contains("getblockcount"));

    let sent: serde_json::Value = serde_json::from_str(&server.requests()[2].body).unwrap();
    assert_eq!(sent["method"], "help");
    assert_eq!(sent["params"], json!(["getblockcount"]));
}
