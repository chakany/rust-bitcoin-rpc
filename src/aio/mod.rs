//! Async client. Requires the `aio` feature.
//!
//! This client is built on [`reqwest`](https://crates.io/crates/reqwest) and
//! runs on a `tokio` reactor: `tokio` is a mandatory transitive dependency
//! (via `reqwest` -> `hyper`), and awaiting its futures from another
//! executor (e.g. `smol`, `async-std`) fails at runtime, not compile time.
//! This crate has no *direct* dependency on `tokio` (it is a dev-dependency
//! only, used by this crate's own tests). To opt out of `tokio` entirely,
//! use the `sync` feature instead.

mod call;
mod methods;

pub use call::{RpcCallAsync, RpcCallAsyncExt};
pub use methods::*;

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde_json::Value;

use crate::config::Config;
use crate::jsonrpc::{Request, parse_reply};
use crate::{Auth, Error, Result};

/// Builds a [`Client`].
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    config: Config,
}

impl ClientBuilder {
    /// Start building a client for the node at `url`, e.g.
    /// `http://127.0.0.1:8332`.
    pub fn new(url: impl Into<String>) -> Self {
        ClientBuilder {
            config: Config::new(url),
        }
    }

    /// Set the credentials. Defaults to [`Auth::None`].
    pub fn auth(mut self, auth: Auth) -> Self {
        self.config.auth = auth;
        self
    }

    /// Set the total per-request timeout, or `None` for no timeout at all.
    /// Defaults to 30 seconds.
    ///
    /// A long-polling method (e.g. `wait_for_new_block`) is cut short by
    /// this timeout well before the RPC-level wait it was asked to make;
    /// pass `None` here to let those calls block for as long as the node
    /// takes to reply.
    pub fn timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.config.timeout = timeout.into();
        self
    }

    /// Cap the size of a reply body, or `None` for no cap. Defaults to
    /// 64 MiB.
    ///
    /// A reply larger than this fails with [`Error::ResponseTooLarge`]
    /// instead of being buffered, so a misbehaving node or proxy cannot make
    /// the client allocate without bound. The default comfortably fits every
    /// response this crate types, including a verbosity-3 `getblock` of a
    /// full block; raise it or pass `None` for a verbose `getrawmempool` on
    /// a very busy node.
    pub fn max_response_size(mut self, limit: impl Into<Option<usize>>) -> Self {
        self.config.max_response_size = limit.into();
        self
    }

    /// Validate the configuration, read the cookie file if one was given, and
    /// construct the client.
    pub fn build(self) -> Result<Client> {
        self.config.validate()?;
        let authorization = self.config.auth.header_value()?;
        // A JSON-RPC endpoint should never redirect; following one could
        // silently re-send credentials to another host.
        let mut builder = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none());
        if let Some(timeout) = self.config.timeout {
            builder = builder.timeout(timeout);
        }
        let http = builder.build().map_err(|e| Error::Config(e.to_string()))?;
        Ok(Client {
            http,
            url: self.config.url,
            authorization,
            max_response_size: self.config.max_response_size,
            next_id: AtomicU64::new(1),
        })
    }
}

/// An async Bitcoin Core RPC client.
///
/// Bring the method traits into scope to use it, e.g.
/// `use bitcoin_rpc::aio::BlockchainRpc;` or `use bitcoin_rpc::prelude::*;`.
#[derive(Debug)]
pub struct Client {
    http: reqwest::Client,
    url: String,
    authorization: Option<String>,
    max_response_size: Option<usize>,
    next_id: AtomicU64,
}

impl Client {
    /// Read the body into memory, stopping as soon as it is known to exceed
    /// the configured cap: up front from `Content-Length` when the node sends
    /// one, otherwise as the chunks arrive.
    async fn read_body(&self, mut response: reqwest::Response) -> Result<Vec<u8>> {
        let Some(limit) = self.max_response_size else {
            return response
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| Error::Transport(e.to_string()));
        };
        if response.content_length().is_some_and(|n| n > limit as u64) {
            return Err(Error::ResponseTooLarge { limit });
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| Error::Transport(e.to_string()))?
        {
            if body.len().saturating_add(chunk.len()) > limit {
                return Err(Error::ResponseTooLarge { limit });
            }
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }
}

impl RpcCallAsync for Client {
    fn call_raw<'a>(
        &'a self,
        method: &'a str,
        params: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            let body = serde_json::to_string(&Request::new(id, method, params))?;

            let mut request = self
                .http
                .post(&self.url)
                .header("Content-Type", "application/json")
                .body(body);
            if let Some(auth) = &self.authorization {
                request = request.header("Authorization", auth);
            }

            // No `error_for_status`: a 500 still carries a usable error body.
            let response = request
                .send()
                .await
                .map_err(|e| Error::Transport(e.to_string()))?;
            let status = response.status().as_u16();
            let bytes = self.read_body(response).await?;

            parse_reply(status, &bytes, id)
        })
    }
}
