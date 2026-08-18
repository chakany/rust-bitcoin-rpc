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
