//! Async client. Requires the `aio` feature.
//!
//! The runtime is the caller's choice; this crate does not depend on `tokio`.

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

    /// Set the total per-request timeout. Defaults to 30 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Validate the configuration, read the cookie file if one was given, and
    /// construct the client.
    pub fn build(self) -> Result<Client> {
        self.config.validate()?;
        let authorization = self.config.auth.header_value()?;
        let http = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .build()
            .map_err(|e| Error::Config(e.to_string()))?;
        Ok(Client {
            http,
            url: self.config.url,
            authorization,
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
    next_id: AtomicU64,
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
            let bytes = response
                .bytes()
                .await
                .map_err(|e| Error::Transport(e.to_string()))?;

            parse_reply(status, &bytes)
        })
    }
}
