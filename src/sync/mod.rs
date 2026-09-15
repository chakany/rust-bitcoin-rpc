//! Blocking client. Requires the `sync` feature.
//!
//! This module pulls in no async runtime: `ureq` has no `tokio` in its
//! dependency graph.

mod call;
mod methods;

pub use call::{RpcCall, RpcCallExt};
pub use methods::*;

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
        // silently re-send credentials to another host. `max_redirects(0)`
        // disables redirect handling entirely (ureq's default is 10).
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(self.config.timeout)
            .http_status_as_error(false)
            .max_redirects(0)
            .build()
            .into();
        Ok(Client {
            agent,
            url: self.config.url,
            authorization,
            max_response_size: self.config.max_response_size,
            next_id: AtomicU64::new(1),
        })
    }
}

/// A blocking Bitcoin Core RPC client.
///
/// Bring the method traits into scope to use it, e.g.
/// `use bitcoin_rpc::sync::BlockchainRpc;` or `use bitcoin_rpc::prelude::*;`.
#[derive(Debug)]
pub struct Client {
    agent: ureq::Agent,
    url: String,
    authorization: Option<String>,
    max_response_size: Option<usize>,
    next_id: AtomicU64,
}

impl RpcCall for Client {
    fn call_raw(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let body = serde_json::to_string(&Request::new(id, method, params))?;

        let mut request = self
            .agent
            .post(&self.url)
            .header("Content-Type", "application/json");
        if let Some(auth) = &self.authorization {
            request = request.header("Authorization", auth);
        }

        let mut resp = request
            .send(&body)
            .map_err(|e| Error::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        // A 500 from the node still carries a usable JSON-RPC error body.
        //
        // `ureq` caps `read_to_vec` at 10 MB unless told otherwise, so the
        // limit is always set explicitly here: to ours, or to "unbounded"
        // when the caller asked for no cap.
        let limit = self.max_response_size;
        let bytes = resp
            .body_mut()
            .with_config()
            .limit(limit.map_or(u64::MAX, |n| n as u64))
            .read_to_vec()
            .map_err(|e| match (e, limit) {
                (ureq::Error::BodyExceedsLimit(_), Some(limit)) => {
                    Error::ResponseTooLarge { limit }
                }
                (e, _) => Error::Transport(e.to_string()),
            })?;

        parse_reply(status, &bytes, id)
    }
}
