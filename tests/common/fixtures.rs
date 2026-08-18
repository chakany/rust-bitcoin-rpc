//! Shared JSON-RPC reply bodies used by both `sync_client.rs` and `aio_client.rs`.
//!
//! Kept here once so client-side round-trip tests for the two transports never
//! drift from each other by hand-copying multi-KB literals.

/// A getblock verbosity-2 reply: the most complex blockchain result, exercising a
/// nested object, an array of objects and an absent optional field.
pub const BLOCK_WITH_TXS_REPLY: &str = r#"{"jsonrpc":"2.0","id":1,"result":{"hash":"00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09","confirmations":799901,"size":285,"strippedsize":285,"weight":1140,"coinbase_tx":{"version":1,"locktime":0,"sequence":4294967295,"coinbase":"04ffff001d0102","witness":"00"},"height":100,"version":1,"versionHex":"00000001","merkleroot":"2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866","tx":[{"txid":"2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866","vin":[],"vout":[],"fee":0.00012345}],"time":1231660825,"mediantime":1231658656,"nonce":2573394689,"bits":"1d00ffff","target":"00000000ffff0000000000000000000000000000000000000000000000000000","difficulty":1.0,"chainwork":"6500650065","nTx":1,"previousblockhash":"000000007bc154e0fa7ea32218a72fe2c1bb9f86cf8c9ebf9a715ed27fdb229a"}}"#;

/// A getmempoolentry reply: the most complex mempool result, exercising the
/// `fees` sub-object and both dependency arrays.
pub const MEMPOOL_ENTRY_REPLY: &str = r#"{"jsonrpc":"2.0","id":1,"result":{"vsize":204,"weight":816,"time":1690000000,"height":800000,"descendantcount":1,"descendantsize":204,"ancestorcount":1,"ancestorsize":204,"chunkweight":816,"wtxid":"2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866","fees":{"base":0.00012345,"modified":0.00012345,"ancestor":0.00012345,"descendant":0.00012345,"chunk":0.00012345},"depends":["4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"],"spentby":[],"bip125-replaceable":true,"unbroadcast":false}}"#;

/// A getpeerinfo reply with one full peer entry: the most complex network
/// result, exercising both `OBJ_DYN` maps and several optional fields.
pub const PEER_INFO_REPLY: &str = r#"{"jsonrpc":"2.0","id":1,"result":[{"id":7,"addr":"192.168.0.6:8333","addrbind":"10.0.0.1:8333","addrlocal":"203.0.113.5:8333","network":"ipv4","mapped_as":12345,"services":"0000000000000409","servicesnames":["NETWORK","WITNESS"],"relaytxes":true,"last_inv_sequence":42,"inv_to_send":3,"lastsend":1690000100,"lastrecv":1690000099,"last_transaction":1690000000,"last_block":1689999000,"bytessent":123456,"bytesrecv":654321,"conntime":1689990000,"timeoffset":-2,"pingtime":0.05,"minping":0.04,"pingwait":0.01,"version":70016,"subver":"/Satoshi:25.0.0/","inbound":false,"bip152_hb_to":true,"bip152_hb_from":false,"presynced_headers":-1,"synced_headers":800000,"synced_blocks":799999,"inflight":[800001,800002],"addr_relay_enabled":true,"addr_processed":100,"addr_rate_limited":2,"permissions":["noban"],"minfeefilter":0.00001000,"bytessent_per_msg":{"ping":32,"verack":24},"bytesrecv_per_msg":{"pong":32},"connection_type":"outbound-full-relay","transport_protocol_type":"v2","session_id":"abcd1234"}]}"#;

/// A getblocktemplate reply: the largest result in the crate, exercising an
/// array of transaction objects, two `OBJ_DYN` maps and an absent optional
/// (`weightlimit`/`signet_challenge`/`default_witness_commitment` are all
/// omitted here, `getmininginfo`'s tests cover the present case).
pub const BLOCK_TEMPLATE_REPLY: &str = r#"{"jsonrpc":"2.0","id":1,"result":{"version":536870912,"rules":["csv","!segwit"],"vbavailable":{"!testdummy":28},"capabilities":["proposal"],"vbrequired":0,"previousblockhash":"00000000000000000002a7c4c1e48d76c5a37902165a270156b7a8d72728a054","transactions":[{"data":"0200000001abcd","txid":"2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866","hash":"2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866","depends":[1],"fee":1000,"sigops":4,"weight":565}],"coinbaseaux":{"flags":""},"coinbasevalue":625000000,"longpollid":"00000000000000000002a7c4c1e48d76c5a37902165a270156b7a8d72728a05412345","target":"0000000000000000000340190000000000000000000000000000000000000","mintime":1690000000,"mutable":["time","transactions","prevblock"],"noncerange":"00000000ffffffff","sigoplimit":80000,"sizelimit":4000000,"curtime":1690000100,"bits":"170d6b91","height":800001}}"#;
