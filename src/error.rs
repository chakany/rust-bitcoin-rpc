//! Error types.

use std::fmt;

/// Result alias used throughout this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// An error returned by the Bitcoin Core node itself.
///
/// `code` is the raw JSON-RPC error code; the values are defined in
/// `src/rpc/protocol.h` of Bitcoin Core (`RPC_*` constants).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcError {
    /// Raw JSON-RPC error code.
    pub code: i32,
    /// Human-readable error message from the node.
    pub message: String,
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RPC error {}: {}", self.code, self.message)
    }
}

/// Anything that can go wrong when talking to a node.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The client was misconfigured: bad URL, unreadable or malformed cookie file.
    Config(String),
    /// The HTTP request failed outright, or a reply arrived that was not a
    /// usable JSON-RPC message; in the latter case the message names the
    /// HTTP status the node returned.
    Transport(String),
    /// The reply body was not the JSON we expected.
    #[cfg(feature = "serde")]
    Json(serde_json::Error),
    /// The node returned a JSON-RPC error.
    Rpc(RpcError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(m) => write!(f, "configuration error: {m}"),
            Error::Transport(m) => write!(f, "transport error: {m}"),
            #[cfg(feature = "serde")]
            Error::Json(e) => write!(f, "json error: {e}"),
            Error::Rpc(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "serde")]
            Error::Json(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_error_displays_code_and_message() {
        let e = Error::Rpc(RpcError {
            code: -8,
            message: "Block not found".into(),
        });
        assert_eq!(e.to_string(), "RPC error -8: Block not found");
    }

    #[test]
    fn config_error_is_prefixed() {
        let e = Error::Config("bad url".into());
        assert_eq!(e.to_string(), "configuration error: bad url");
    }
}
