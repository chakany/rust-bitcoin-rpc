//! A JSON-RPC client for Bitcoin Core v31.1.
//!
//! Enable the `sync` feature for a blocking client or the `aio` feature for an
//! async one. Neither is enabled by default.
#![warn(missing_docs)]

mod error;
mod auth;

pub mod types;

pub use error::{Error, RpcError, Result};
pub use auth::Auth;
