#![cfg(feature = "serde")]

use bitcoin_rpc::types::*;
use serde_json::json;

#[test]
fn get_blockchain_info_full() {
    let v = json!({
        "chain": "main", "blocks": 800000, "headers": 800000,
        "bestblockhash": "0000000000000000000",
        "bits": "17034219", "target": "000000000000000000034219",
        "difficulty": 53911173001054.59, "time": 1690000000,
        "mediantime": 1689999000, "verificationprogress": 0.9999,
        "initialblockdownload": false, "chainwork": "00000000000000abc",
        "size_on_disk": 570000000000_u64, "pruned": true,
        "pruneheight": 700000, "automatic_pruning": true,
        "prune_target_size": 550000000_u64, "signet_challenge": "51",
        "warnings": ["unknown new rules activated"]
    });
    let info: GetBlockchainInfo = serde_json::from_value(v).unwrap();
    assert_eq!(info.chain, "main");
    assert_eq!(info.best_block_hash, "0000000000000000000");
    assert_eq!(info.median_time, 1689999000);
    assert_eq!(info.verification_progress, 0.9999);
    assert!(!info.initial_block_download);
    assert_eq!(info.size_on_disk, 570000000000);
    assert_eq!(info.prune_height, Some(700000));
    assert_eq!(info.automatic_pruning, Some(true));
    assert_eq!(info.prune_target_size, Some(550000000));
    assert_eq!(info.signet_challenge.as_deref(), Some("51"));
    assert_eq!(info.warnings, vec!["unknown new rules activated"]);
}

#[test]
fn get_blockchain_info_minimal_and_forward_compatible() {
    let v = json!({
        "chain": "regtest", "blocks": 0, "headers": 0,
        "bestblockhash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
        "bits": "207fffff", "target": "7fffff0000000000",
        "difficulty": 4.656542373871732e-10, "time": 1296688602,
        "mediantime": 1296688602, "verificationprogress": 1.0,
        "initialblockdownload": true, "chainwork": "02",
        "size_on_disk": 293, "pruned": false, "warnings": [],
        "some_field_from_a_future_release": 1
    });
    let info: GetBlockchainInfo = serde_json::from_value(v).unwrap();
    assert_eq!(info.chain, "regtest");
    assert_eq!(info.prune_height, None);
    assert_eq!(info.automatic_pruning, None);
    assert_eq!(info.prune_target_size, None);
    assert_eq!(info.signet_challenge, None);
    assert!(info.warnings.is_empty());
}

#[test]
fn block_header_full() {
    let v = json!({
        "hash": "00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09",
        "confirmations": 799901, "height": 100, "version": 536870912,
        "versionHex": "20000000",
        "merkleroot": "2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866",
        "time": 1231660825, "mediantime": 1231658656, "nonce": 2573394689_u64,
        "bits": "1d00ffff", "target": "00000000ffff0000000000000000000000000000000000000000000000000000",
        "difficulty": 1.0,
        "chainwork": "0000000000000000000000000000000000000000000000000000006500650065",
        "nTx": 1,
        "previousblockhash": "000000007bc154e0fa7ea32218a72fe2c1bb9f86cf8c9ebf9a715ed27fdb229a",
        "nextblockhash": "00000000fe0b4e0d84a91ad24b23d5b45cd0e21ac4cd0d0d5e13c6a2f4a0e17c"
    });
    let header: BlockHeader = serde_json::from_value(v).unwrap();
    assert_eq!(header.confirmations, 799901);
    assert_eq!(header.height, 100);
    assert_eq!(header.version, 536870912);
    assert_eq!(header.version_hex, "20000000");
    assert_eq!(header.median_time, 1231658656);
    assert_eq!(header.nonce, 2573394689);
    assert_eq!(header.n_tx, 1);
    assert!(header.previous_block_hash.is_some());
    assert!(header.next_block_hash.is_some());
}

#[test]
fn block_header_minimal_and_forward_compatible() {
    let v = json!({
        "hash": "00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09",
        "confirmations": -1, "height": 100, "version": 536870912,
        "versionHex": "20000000",
        "merkleroot": "2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866",
        "time": 1231660825, "mediantime": 1231658656, "nonce": 0,
        "bits": "1d00ffff", "target": "00000000ffff0000000000000000000000000000000000000000000000000000",
        "difficulty": 1.0, "chainwork": "65", "nTx": 1,
        "some_field_from_a_future_release": true
    });
    let header: BlockHeader = serde_json::from_value(v).unwrap();
    // A block off the main chain reports -1 confirmations, so the field is signed.
    assert_eq!(header.confirmations, -1);
    assert_eq!(header.previous_block_hash, None);
    assert_eq!(header.next_block_hash, None);
}

#[test]
fn block_full() {
    let v = json!({
        "hash": "00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09",
        "confirmations": 799901, "size": 285, "strippedsize": 285, "weight": 1140,
        "coinbase_tx": {
            "version": 1, "locktime": 0, "sequence": 4294967295_u64,
            "coinbase": "04ffff001d0102", "witness": "0000000000000000000000000000000000000000000000000000000000000000"
        },
        "height": 100, "version": 536870912, "versionHex": "20000000",
        "merkleroot": "2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866",
        "tx": ["2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866"],
        "time": 1231660825, "mediantime": 1231658656, "nonce": 2573394689_u64,
        "bits": "1d00ffff", "target": "00000000ffff0000000000000000000000000000000000000000000000000000",
        "difficulty": 1.0, "chainwork": "6500650065", "nTx": 1,
        "previousblockhash": "000000007bc154e0fa7ea32218a72fe2c1bb9f86cf8c9ebf9a715ed27fdb229a",
        "nextblockhash": "00000000fe0b4e0d84a91ad24b23d5b45cd0e21ac4cd0d0d5e13c6a2f4a0e17c"
    });
    let block: Block = serde_json::from_value(v).unwrap();
    assert_eq!(block.size, 285);
    assert_eq!(block.stripped_size, 285);
    assert_eq!(block.weight, 1140);
    assert_eq!(block.coinbase_tx.version, 1);
    assert_eq!(block.coinbase_tx.sequence, 4294967295);
    assert_eq!(block.coinbase_tx.coinbase, "04ffff001d0102");
    assert!(block.coinbase_tx.witness.is_some());
    assert_eq!(
        block.tx,
        vec!["2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866"]
    );
    assert_eq!(block.n_tx, 1);
    assert!(block.next_block_hash.is_some());
}

#[test]
fn block_minimal_and_forward_compatible() {
    let v = json!({
        "hash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
        "confirmations": 1, "size": 285, "strippedsize": 285, "weight": 1140,
        "coinbase_tx": { "version": 1, "locktime": 0, "sequence": 4294967295_u64, "coinbase": "51" },
        "height": 0, "version": 1, "versionHex": "00000001",
        "merkleroot": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
        "tx": [], "time": 1296688602, "mediantime": 1296688602, "nonce": 2,
        "bits": "207fffff", "target": "7fffff0000000000",
        "difficulty": 4.656542373871732e-10, "chainwork": "02", "nTx": 1,
        "some_field_from_a_future_release": ["anything"]
    });
    let block: Block = serde_json::from_value(v).unwrap();
    assert_eq!(block.height, 0);
    assert_eq!(block.coinbase_tx.witness, None);
    assert!(block.tx.is_empty());
    assert_eq!(block.previous_block_hash, None);
    assert_eq!(block.next_block_hash, None);
}

#[test]
fn block_with_txs_full() {
    let v = json!({
        "hash": "00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09",
        "confirmations": 799901, "size": 285, "strippedsize": 285, "weight": 1140,
        "coinbase_tx": {
            "version": 1, "locktime": 0, "sequence": 4294967295_u64,
            "coinbase": "04ffff001d0102", "witness": "00"
        },
        "height": 100, "version": 536870912, "versionHex": "20000000",
        "merkleroot": "2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866",
        "tx": [
            {
                "txid": "2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866",
                "hash": "2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866",
                "version": 1, "size": 204, "vsize": 204, "weight": 816, "locktime": 0,
                "vin": [], "vout": [], "hex": "0100000001",
                "fee": 0.00012345
            }
        ],
        "time": 1231660825, "mediantime": 1231658656, "nonce": 2573394689_u64,
        "bits": "1d00ffff", "target": "00000000ffff0000000000000000000000000000000000000000000000000000",
        "difficulty": 1.0, "chainwork": "6500650065", "nTx": 1,
        "previousblockhash": "000000007bc154e0fa7ea32218a72fe2c1bb9f86cf8c9ebf9a715ed27fdb229a",
        "nextblockhash": "00000000fe0b4e0d84a91ad24b23d5b45cd0e21ac4cd0d0d5e13c6a2f4a0e17c"
    });
    let block: BlockWithTxs = serde_json::from_value(v).unwrap();
    assert_eq!(block.height, 100);
    assert_eq!(block.coinbase_tx.locktime, 0);
    assert_eq!(block.tx.len(), 1);
    // `fee` is a JSON number even though Core documents it as STR_AMOUNT.
    assert_eq!(block.tx[0].fee, Some(0.00012345));
    assert!(block.previous_block_hash.is_some());
}

#[test]
fn block_with_txs_minimal_and_forward_compatible() {
    let v = json!({
        "hash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
        "confirmations": 1, "size": 285, "strippedsize": 285, "weight": 1140,
        "coinbase_tx": { "version": 1, "locktime": 0, "sequence": 4294967295_u64, "coinbase": "51" },
        "height": 0, "version": 1, "versionHex": "00000001",
        "merkleroot": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
        "tx": [{ "txid": "4a5e1e", "vin": [], "vout": [] }],
        "time": 1296688602, "mediantime": 1296688602, "nonce": 2,
        "bits": "207fffff", "target": "7fffff0000000000",
        "difficulty": 4.656542373871732e-10, "chainwork": "02", "nTx": 1,
        "some_field_from_a_future_release": 1
    });
    let block: BlockWithTxs = serde_json::from_value(v).unwrap();
    assert_eq!(block.tx.len(), 1);
    // Blocks whose undo data is unavailable (e.g. pruned) carry no per-tx fee.
    assert_eq!(block.tx[0].fee, None);
    assert_eq!(block.next_block_hash, None);
}

#[test]
fn chain_tip_full() {
    let v = json!({
        "height": 800000,
        "hash": "00000000000000000002a7c4c1e48d76c5a37902165a270156b7a8d72728a054",
        "branchlen": 0,
        "status": "active"
    });
    let tip: ChainTip = serde_json::from_value(v).unwrap();
    assert_eq!(tip.height, 800000);
    assert_eq!(tip.branch_len, 0);
    assert_eq!(tip.status, "active");
}

#[test]
fn chain_tip_forward_compatible() {
    let v = json!({
        "height": 799999,
        "hash": "000000000000000000019dbb35b19b8e0a2f4a6be4b78f6b47cee0d0b2f4f9a1",
        "branchlen": 2,
        "status": "valid-fork",
        "some_field_from_a_future_release": "x"
    });
    let tip: ChainTip = serde_json::from_value(v).unwrap();
    assert_eq!(tip.branch_len, 2);
    assert_eq!(tip.status, "valid-fork");
}

#[test]
fn deployment_info_full() {
    let v = json!({
        "hash": "00000000000000000002a7c4c1e48d76c5a37902165a270156b7a8d72728a054",
        "height": 800000,
        "script_flags": ["P2SH", "WITNESS", "TAPROOT"],
        "deployments": {
            "taproot": {
                "type": "bip9",
                "height": 709632,
                "active": true,
                "bip9": {
                    "bit": 2,
                    "start_time": 1619222400,
                    "timeout": 1628640000,
                    "min_activation_height": 709632,
                    "status": "active",
                    "since": 709632,
                    "status_next": "active",
                    "statistics": {
                        "period": 2016,
                        "threshold": 1815,
                        "elapsed": 2016,
                        "count": 1916,
                        "possible": true
                    },
                    "signalling": "#####--#####"
                }
            }
        }
    });
    let info: DeploymentInfo = serde_json::from_value(v).unwrap();
    assert_eq!(info.height, 800000);
    assert_eq!(info.script_flags, vec!["P2SH", "WITNESS", "TAPROOT"]);
    let taproot = &info.deployments["taproot"];
    assert_eq!(taproot.deployment_type, "bip9");
    assert_eq!(taproot.height, Some(709632));
    assert!(taproot.active);
    let bip9 = taproot.bip9.as_ref().unwrap();
    assert_eq!(bip9.bit, Some(2));
    assert_eq!(bip9.start_time, 1619222400);
    assert_eq!(bip9.min_activation_height, 709632);
    assert_eq!(bip9.status_next, "active");
    assert_eq!(bip9.signalling.as_deref(), Some("#####--#####"));
    let stats = bip9.statistics.as_ref().unwrap();
    assert_eq!(stats.period, 2016);
    assert_eq!(stats.threshold, Some(1815));
    assert_eq!(stats.count, 1916);
    assert_eq!(stats.possible, Some(true));
}

#[test]
fn deployment_info_minimal_and_forward_compatible() {
    let v = json!({
        "hash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
        "height": 0,
        "script_flags": [],
        "deployments": {
            "segwit": { "type": "buried", "active": true },
            "testdummy": {
                "type": "bip9",
                "active": false,
                "bip9": {
                    "start_time": 0,
                    "timeout": 9223372036854775807_i64,
                    "min_activation_height": 0,
                    "status": "defined",
                    "since": 0,
                    "status_next": "defined"
                }
            }
        },
        "some_field_from_a_future_release": {}
    });
    let info: DeploymentInfo = serde_json::from_value(v).unwrap();
    assert!(info.script_flags.is_empty());
    let segwit = &info.deployments["segwit"];
    assert_eq!(segwit.deployment_type, "buried");
    assert_eq!(segwit.height, None);
    assert_eq!(segwit.bip9, None);
    let bip9 = info.deployments["testdummy"].bip9.as_ref().unwrap();
    assert_eq!(bip9.bit, None);
    assert_eq!(bip9.timeout, i64::MAX);
    assert_eq!(bip9.statistics, None);
    assert_eq!(bip9.signalling, None);
}

#[test]
fn tx_out_full() {
    let v = json!({
        "bestblock": "00000000000000000002a7c4c1e48d76c5a37902165a270156b7a8d72728a054",
        "confirmations": 42,
        "value": 0.00012345,
        "scriptPubKey": {
            "asm": "OP_DUP OP_HASH160 0000 OP_EQUALVERIFY OP_CHECKSIG",
            "desc": "addr(bc1qexample)#checksum",
            "hex": "76a914000088ac",
            "type": "pubkeyhash",
            "address": "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH"
        },
        "coinbase": true
    });
    let out: TxOut = serde_json::from_value(v).unwrap();
    assert_eq!(out.confirmations, 42);
    // STR_AMOUNT is an unquoted JSON number, hence f64.
    assert_eq!(out.value, 0.00012345);
    assert_eq!(out.script_pub_key.script_type, "pubkeyhash");
    assert_eq!(out.script_pub_key.hex, "76a914000088ac");
    assert_eq!(
        out.script_pub_key.address.as_deref(),
        Some("1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH")
    );
    assert!(out.coinbase);
    assert!(out.best_block.starts_with("00000000"));
}

#[test]
fn tx_out_minimal_and_forward_compatible() {
    let v = json!({
        "bestblock": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
        "confirmations": 0,
        "value": 50.0,
        "scriptPubKey": {
            "asm": "OP_RETURN",
            "desc": "raw(6a)#checksum",
            "hex": "6a",
            "type": "nulldata",
            "some_field_from_a_future_release": 1
        },
        "coinbase": false,
        "another_field_from_a_future_release": null
    });
    let out: TxOut = serde_json::from_value(v).unwrap();
    assert_eq!(out.confirmations, 0);
    assert_eq!(out.value, 50.0);
    assert_eq!(out.script_pub_key.address, None);
    assert!(!out.coinbase);
}

#[test]
fn tx_out_miss_deserializes_to_none() {
    // `gettxout` answers JSON null when the output is not in the UTXO set.
    let out: Option<TxOut> = serde_json::from_value(json!(null)).unwrap();
    assert_eq!(out, None);
}

#[test]
fn block_hash_and_height_full() {
    let v = json!({
        "hash": "00000000000000000002a7c4c1e48d76c5a37902165a270156b7a8d72728a054",
        "height": 800000
    });
    let tip: BlockHashAndHeight = serde_json::from_value(v).unwrap();
    assert_eq!(tip.height, 800000);
    assert!(tip.hash.starts_with("000000000000"));
}

#[test]
fn block_hash_and_height_forward_compatible() {
    let v = json!({
        "hash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
        "height": 0,
        "some_field_from_a_future_release": 1
    });
    let tip: BlockHashAndHeight = serde_json::from_value(v).unwrap();
    assert_eq!(tip.height, 0);
}

#[test]
fn get_mempool_info_full() {
    let v = json!({
        "loaded": true, "size": 120, "bytes": 45000, "usage": 987654,
        "total_fee": 0.01234567, "maxmempool": 300000000_u64,
        "mempoolminfee": 0.00001000, "minrelaytxfee": 0.00001000,
        "incrementalrelayfee": 0.00001000, "unbroadcastcount": 3,
        "permitbaremultisig": true, "maxdatacarriersize": 83,
        "limitclustercount": 500, "limitclustersize": 101000,
        "optimal": true
    });
    let info: GetMempoolInfo = serde_json::from_value(v).unwrap();
    assert!(info.loaded);
    assert_eq!(info.size, 120);
    assert_eq!(info.bytes, 45000);
    assert_eq!(info.usage, 987654);
    // STR_AMOUNT is an unquoted JSON number, hence f64.
    assert_eq!(info.total_fee, 0.01234567);
    assert_eq!(info.max_mempool, 300000000);
    assert_eq!(info.mempool_min_fee, 0.00001000);
    assert_eq!(info.min_relay_tx_fee, 0.00001000);
    assert_eq!(info.incremental_relay_fee, 0.00001000);
    assert_eq!(info.unbroadcast_count, 3);
    assert!(info.permit_bare_multisig);
    assert_eq!(info.max_datacarrier_size, 83);
    assert_eq!(info.limit_cluster_count, 500);
    assert_eq!(info.limit_cluster_size, 101000);
    assert!(info.optimal);
}

#[test]
fn get_mempool_info_forward_compatible() {
    // Every field is required in `getmempoolinfo`'s RPCResult, so this test's
    // forward-compatibility burden falls entirely on the unknown field, plus
    // proving the deprecated `fullrbf` field (present on a real node) is
    // tolerated even though `GetMempoolInfo` has no field for it.
    let v = json!({
        "loaded": false, "size": 0, "bytes": 0, "usage": 0,
        "total_fee": 0.0, "maxmempool": 300000000_u64,
        "mempoolminfee": 0.00001000, "minrelaytxfee": 0.00001000,
        "incrementalrelayfee": 0.00001000, "unbroadcastcount": 0,
        "permitbaremultisig": false, "maxdatacarriersize": 0,
        "limitclustercount": 0, "limitclustersize": 0,
        "optimal": false,
        "fullrbf": true,
        "some_field_from_a_future_release": 1
    });
    let info: GetMempoolInfo = serde_json::from_value(v).unwrap();
    assert!(!info.loaded);
    assert_eq!(info.size, 0);
    assert!(!info.optimal);
}

#[test]
fn mempool_entry_full() {
    let v = json!({
        "vsize": 204, "weight": 816, "time": 1690000000, "height": 800000,
        "descendantcount": 2, "descendantsize": 408,
        "ancestorcount": 1, "ancestorsize": 204,
        "chunkweight": 816,
        "wtxid": "2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866",
        "fees": {
            "base": 0.00012345, "modified": 0.00012400,
            "ancestor": 0.00012345, "descendant": 0.00024690,
            "chunk": 0.00012345
        },
        "depends": ["4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"],
        "spentby": ["9b1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b1"],
        "bip125-replaceable": true,
        "unbroadcast": false
    });
    let entry: MempoolEntry = serde_json::from_value(v).unwrap();
    assert_eq!(entry.vsize, 204);
    assert_eq!(entry.weight, 816);
    assert_eq!(entry.time, 1690000000);
    assert_eq!(entry.height, 800000);
    assert_eq!(entry.descendant_count, 2);
    assert_eq!(entry.descendant_size, 408);
    assert_eq!(entry.ancestor_count, 1);
    assert_eq!(entry.ancestor_size, 204);
    assert_eq!(entry.chunk_weight, 816);
    assert_eq!(
        entry.wtxid,
        "2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866"
    );
    // STR_AMOUNT fields are unquoted JSON numbers, hence f64.
    assert_eq!(entry.fees.base, 0.00012345);
    assert_eq!(entry.fees.modified, 0.00012400);
    assert_eq!(entry.fees.ancestor, 0.00012345);
    assert_eq!(entry.fees.descendant, 0.00024690);
    assert_eq!(entry.fees.chunk, 0.00012345);
    assert_eq!(entry.depends.len(), 1);
    assert_eq!(entry.spent_by.len(), 1);
    assert!(!entry.unbroadcast);
}

#[test]
fn mempool_entry_minimal_and_forward_compatible() {
    // Every field is required in `MempoolEntryDescription`, so this test's
    // forward-compatibility burden falls on the unknown field, plus proving
    // the deprecated `bip125-replaceable` field is simply absent here and
    // that omitting it from `MempoolEntry` does not break deserialization.
    let v = json!({
        "vsize": 110, "weight": 440, "time": 1600000000, "height": 700000,
        "descendantcount": 1, "descendantsize": 110,
        "ancestorcount": 1, "ancestorsize": 110,
        "chunkweight": 440,
        "wtxid": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
        "fees": {
            "base": 0.0, "modified": 0.0, "ancestor": 0.0,
            "descendant": 0.0, "chunk": 0.0
        },
        "depends": [], "spentby": [],
        "unbroadcast": true,
        "some_field_from_a_future_release": "x"
    });
    let entry: MempoolEntry = serde_json::from_value(v).unwrap();
    assert_eq!(entry.vsize, 110);
    assert!(entry.depends.is_empty());
    assert!(entry.spent_by.is_empty());
    assert!(entry.unbroadcast);
    assert_eq!(entry.fees.base, 0.0);
}

#[test]
fn get_raw_mempool_sequence_full() {
    let v = json!({
        "txids": ["2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866"],
        "mempool_sequence": 12345_u64
    });
    let seq: GetRawMempoolSequence = serde_json::from_value(v).unwrap();
    assert_eq!(seq.txids.len(), 1);
    assert_eq!(
        seq.txids[0],
        "2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866"
    );
    assert_eq!(seq.mempool_sequence, 12345);
}

#[test]
fn get_raw_mempool_sequence_forward_compatible() {
    let v = json!({
        "txids": [],
        "mempool_sequence": 0,
        "some_field_from_a_future_release": 1
    });
    let seq: GetRawMempoolSequence = serde_json::from_value(v).unwrap();
    assert!(seq.txids.is_empty());
    assert_eq!(seq.mempool_sequence, 0);
}
