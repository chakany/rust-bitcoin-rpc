//! Shared JSON-RPC reply bodies used by both `sync_client.rs` and `aio_client.rs`.
//!
//! Kept here once so client-side round-trip tests for the two transports never
//! drift from each other by hand-copying multi-KB literals.

/// A getblock verbosity-2 reply: the most complex blockchain result, exercising a
/// nested object, an array of objects and an absent optional field.
pub const BLOCK_WITH_TXS_REPLY: &str = r#"{"jsonrpc":"2.0","id":1,"result":{"hash":"00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09","confirmations":799901,"size":285,"strippedsize":285,"weight":1140,"coinbase_tx":{"version":1,"locktime":0,"sequence":4294967295,"coinbase":"04ffff001d0102","witness":"00"},"height":100,"version":1,"versionHex":"00000001","merkleroot":"2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866","tx":[{"txid":"2d05f0c9c3e1c226e63b5fac240137687544cf631cd616fd34fd188fc9020866","vin":[],"vout":[],"fee":0.00012345}],"time":1231660825,"mediantime":1231658656,"nonce":2573394689,"bits":"1d00ffff","target":"00000000ffff0000000000000000000000000000000000000000000000000000","difficulty":1.0,"chainwork":"6500650065","nTx":1,"previousblockhash":"000000007bc154e0fa7ea32218a72fe2c1bb9f86cf8c9ebf9a715ed27fdb229a"}}"#;
