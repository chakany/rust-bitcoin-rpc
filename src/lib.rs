//! A JSON-RPC client for Bitcoin Core v31.1.
//!
//! Enable the `sync` feature for a blocking client or the `aio` feature for an
//! async one. Neither is enabled by default.
#![warn(missing_docs)]

mod auth;
mod error;

#[cfg(any(feature = "sync", feature = "aio"))]
mod config;
#[cfg(any(feature = "sync", feature = "aio"))]
mod jsonrpc;
#[cfg(any(feature = "sync", feature = "aio"))]
pub mod params;

pub mod types;

#[cfg(feature = "sync")]
pub mod sync;

#[cfg(feature = "aio")]
pub mod aio;

pub use auth::Auth;
pub use error::{Error, Result, RpcError};

/// Everything you normally want in scope.
///
/// The sync and async method traits share names, so importing both preludes'
/// worth of traits at once is ambiguous by design — import
/// `prelude::sync::*` or `prelude::aio::*` when both features are on.
pub mod prelude {
    /// Blocking traits.
    #[cfg(feature = "sync")]
    pub mod sync {
        pub use crate::params::positional;
        pub use crate::sync::{
            BlockchainRpc, Client, ClientBuilder, ControlRpc, FeeRpc, MempoolRpc, MiningRpc,
            NetworkRpc, RawTransactionsRpc, RpcCall, RpcCallExt, UtilRpc,
        };
    }

    /// Async traits.
    #[cfg(feature = "aio")]
    pub mod aio {
        pub use crate::aio::{
            BlockchainRpc, Client, ClientBuilder, ControlRpc, FeeRpc, MempoolRpc, MiningRpc,
            NetworkRpc, RawTransactionsRpc, RpcCallAsync, RpcCallAsyncExt, UtilRpc,
        };
        pub use crate::params::positional;
    }

    pub use crate::{Auth, Error, Result, RpcError};

    #[cfg(all(feature = "sync", not(feature = "aio")))]
    pub use sync::*;

    #[cfg(all(feature = "aio", not(feature = "sync")))]
    pub use aio::*;
}

// Wires README.md's code samples in as doctests, so `cargo test
// --features sync,aio --doc` fails if they stop compiling. Only makes sense
// with both `sync` and `aio` on, since the README shows both clients.
#[cfg_attr(
    all(feature = "sync", feature = "aio"),
    doc = include_str!("../README.md")
)]
mod readme_doctest {}
