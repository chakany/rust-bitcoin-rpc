# bitcoind-rpc-client

A JSON-RPC client for Bitcoin Core v31.1, with blocking and async
implementations that share one typed method set.

## Install

```toml
[dependencies]
bitcoind-rpc-client = { version = "0.1", features = ["sync"] }  # or ["aio"], or both
```

The library is imported as `bitcoin_rpc`, not as `bitcoind_rpc_client` — the
package name is the long one, the crate you `use` is the short one.

## Features

Nothing is enabled by default; pick the client(s) you need.

| Feature | What it does |
| --- | --- |
| `serde` | `Serialize`/`Deserialize` on the crate's types. Pulled in automatically by `sync` and `aio`; on its own it adds no client. |
| `sync` | A blocking client built on [`ureq`]. Links no async runtime. |
| `aio` | An async client built on [`reqwest`]. Because of that, it runs on a `tokio` reactor: `tokio` is a mandatory transitive dependency (via `reqwest` → `hyper`) and awaiting the client's futures from another executor (e.g. `smol`, `async-std`) fails at runtime. This crate has no *direct* dependency on `tokio` — it's a dev-dependency only, used by this crate's own tests. If you need to opt out of `tokio` entirely, use `sync` instead. |
| `tls` | TLS support for whichever client(s) are enabled. Rustls only — `native-tls`/OpenSSL is never enabled under any feature combination. `aio` + `tls` needs `cmake` and a C compiler available at build time, to build `aws-lc-sys`. |

[`ureq`]: https://crates.io/crates/ureq
[`reqwest`]: https://crates.io/crates/reqwest

> [!NOTE]
> With `aio` + `tls`, your lockfile will show `openssl-probe` (pulled in via
> `reqwest` → `rustls-platform-verifier` → `rustls-native-certs`). Despite the
> name, it is not a TLS backend: it's a small pure-Rust crate, published by the
> `rustls` org, that only checks well-known filesystem paths for a CA bundle
> (the same paths OpenSSL conventionally uses) so rustls can load one. It has
> no build script and no `-sys` dependency, and never links against the
> OpenSSL library. Its presence does not contradict the "no OpenSSL" claim
> above.

## Quickstart: sync

```rust,no_run
use bitcoin_rpc::prelude::{sync::*, Auth};

# fn main() -> bitcoin_rpc::Result<()> {
let client = ClientBuilder::new("http://127.0.0.1:8332")
    .auth(Auth::cookie_file("/home/user/.bitcoin/.cookie"))
    .build()?;

let info = client.get_blockchain_info()?;
println!("{} blocks on {}", info.blocks, info.chain);
# Ok(())
# }
```

## Quickstart: async

```rust,no_run
use bitcoin_rpc::prelude::{aio::*, Auth};

# #[tokio::main]
# async fn main() -> bitcoin_rpc::Result<()> {
let client = ClientBuilder::new("http://127.0.0.1:8332")
    .auth(Auth::cookie_file("/home/user/.bitcoin/.cookie"))
    .build()?;

let info = client.get_blockchain_info().await?;
println!("{} blocks on {}", info.blocks, info.chain);
# Ok(())
# }
```

These two snippets really do compile: with `sync` and `aio` both enabled,
this whole README is pulled into the crate root as a doctest (see
`src/lib.rs`), so `cargo test --features sync,aio --doc` fails if either one
stops compiling. Beyond compiling, the builder-and-`auth`-and-`build` pattern
is exactly what `tests/sync_client.rs` and `tests/aio_client.rs` use
throughout, `Auth::cookie_file` is unit-tested in `src/auth.rs` and used live
in `examples/btc-cli.rs`, and the `blocks`/`chain` fields on the
`get_blockchain_info` result are pinned by the `get_blockchain_info_full`
fixture test in `tests/serde_fixtures.rs`. Only the network round-trip to a
real node is untested — see [Scope](#scope).

## Adding your own RPC

This crate ships 44 typed methods (see [Scope](#scope) below) but not every
RPC Bitcoin Core exposes. Reaching anything else — the wallet RPCs, the
hidden/regtest-only RPCs, or a method a future Core release adds — means
writing your own extension trait over `RpcCall` (sync) or `RpcCallAsync`
(async). This is exactly the mechanism the crate's own typed methods are
built on, and it is exercised end-to-end in `tests/extension_trait.rs`:

```rust
use bitcoin_rpc::Result;
use bitcoin_rpc::sync::{RpcCall, RpcCallExt};
use serde_json::json;

/// A response type a user of this crate would define themselves.
#[derive(Debug, serde::Deserialize)]
pub struct OrphanTx {
    txid: String,
}

/// Written exactly as a downstream crate would, for an RPC we do not ship.
pub trait OrphanRpc: RpcCall {
    fn get_orphan_txs(&self) -> Result<Vec<OrphanTx>> {
        self.call("getorphantxs", json!([2]))
    }
}
impl<T: RpcCall + ?Sized> OrphanRpc for T {}
```

The async version is the same shape, using `RpcCallAsync`/`RpcCallAsyncExt`
and returning `impl Future` instead of a plain `Result`:

```rust
# /// A response type a user of this crate would define themselves.
# #[derive(Debug, serde::Deserialize)]
# pub struct OrphanTx {
#     txid: String,
# }
use bitcoin_rpc::Result;
use bitcoin_rpc::aio::{RpcCallAsync, RpcCallAsyncExt};
use serde_json::json;
use std::future::Future;

/// Written exactly as a downstream crate would, for an RPC we do not ship.
pub trait OrphanRpc: RpcCallAsync {
    fn get_orphan_txs(&self) -> impl Future<Output = Result<Vec<OrphanTx>>> + Send + '_ {
        self.call("getorphantxs", json!([2]))
    }
}
impl<T: RpcCallAsync + ?Sized> OrphanRpc for T {}
```

Both forms use the same blanket-`impl`-over-`RpcCall`/`RpcCallAsync`
mechanism the crate's own shipped traits use, so they compose with those
traits on the same client value rather than conflicting with them — proven
for the sync side by
`user_extension_composes_with_a_shipped_trait_on_the_same_client` in
`tests/extension_trait.rs`.

## Scope

54 typed methods across eight traits: `BlockchainRpc` (15),
`RawTransactionsRpc` (15), `NetworkRpc` (6), `MempoolRpc` (5), `MiningRpc` (5),
`ControlRpc` (4), `UtilRpc` (3), `FeeRpc` (1) — over 48 distinct RPC commands.
The surplus of 6 comes from four commands whose result shape depends on a
verbosity or mode argument, so each is split into a separate typed method: `getblock` (x3:
`get_block_hex`, `get_block`, `get_block_with_txs`), `getrawmempool` (x3:
`get_raw_mempool`, `get_raw_mempool_verbose`, `get_raw_mempool_with_sequence`),
`getblockheader` (x2: `get_block_header`, `get_block_header_hex`), and
`getrawtransaction` (x2: `get_raw_transaction`, `get_raw_transaction_hex`).

`RawTransactionsRpc` covers the whole wallet-free PSBT family: `createpsbt`,
`decodepsbt`, `analyzepsbt`, `finalizepsbt`, `descriptorprocesspsbt`,
`combinepsbt`, `joinpsbts`, `converttopsbt` and `utxoupdatepsbt`, with
`deriveaddresses` on `UtilRpc`. `decodepsbt`'s result is modelled in full,
including the Taproot fields and the BIP 373 MuSig2 fields added in v31.

Not covered, deliberately:

- No wallet RPCs (everything under Bitcoin Core's `wallet` category).
- Three non-wallet `rawtransactions` RPCs: `decodescript`,
  `combinerawtransaction`, and `signrawtransactionwithkey` — the last takes
  raw private keys as an argument, which this crate deliberately has no API
  for.
- No hidden or regtest-only RPCs (e.g. `generatetoaddress`, `invalidateblock`).
- No request batching — one JSON-RPC request per call.
- Deprecated arguments are omitted from method signatures, with one
  exception: `warnings` accepts both Core's modern array form and its legacy
  bare-string form (emitted by a node run with `-deprecatedrpc=warnings`),
  because rejecting the legacy form would otherwise break
  `getblockchaininfo` outright against such a node.

Use the extension-trait pattern above for anything on the "not covered" list.

This crate has been verified against the Bitcoin Core v31.1 source and
against a mock HTTP server (see `tests/`). The PSBT result types are
additionally checked against payloads captured from a live bitcoind v31.1 on
regtest (`tests/data/`): each one is deserialized and re-serialized, so a
field this crate failed to model would show up as missing. The remaining
methods have not yet been exercised against a live node's happy path.

## Errors

Every fallible call returns `bitcoin_rpc::Result<T>`, an alias for
`Result<T, Error>`. `Error` has four variants:

- `Config` — the client was misconfigured: a bad URL, or an unreadable or
  malformed cookie file.
- `Transport` — the HTTP request failed, or the node's HTTP-level response
  could not be turned into a JSON-RPC reply (for example, a `401` from a
  node that rejected the supplied credentials arrives this way, naming the
  status in the message).
- `Json` — the reply body was not the JSON expected at the JSON-RPC layer.
- `Rpc` — the node executed the request and returned a JSON-RPC error. Its
  `code` is Core's raw `RPC_*` constant from `src/rpc/protocol.h`.

`Error` is `#[non_exhaustive]`, so a future release can add variants without
that being a breaking change. Downstream `match` expressions must include a
wildcard arm (`_ => ...`) — matching all four variants today and nothing else
will fail to compile.

## License

MIT, Copyright (c) 2026 Jakub Trnka. See [`LICENSE`](./LICENSE).
