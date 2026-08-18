//! RPC authentication.

use std::path::{Path, PathBuf};

#[cfg(any(feature = "sync", feature = "aio"))]
use crate::{Error, Result};

/// How to authenticate against the node's RPC interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Auth {
    /// Send no `Authorization` header.
    None,
    /// HTTP Basic auth with an explicit username and password
    /// (`-rpcuser`/`-rpcpassword`).
    UserPass(String, String),
    /// Read `user:password` from Bitcoin Core's `.cookie` file.
    CookieFile(PathBuf),
}

impl Auth {
    /// Authenticate with an explicit username and password.
    pub fn user_pass(user: impl Into<String>, pass: impl Into<String>) -> Self {
        Auth::UserPass(user.into(), pass.into())
    }

    /// Authenticate using Bitcoin Core's `.cookie` file, normally found at
    /// `<datadir>/.cookie`.
    pub fn cookie_file(path: impl AsRef<Path>) -> Self {
        Auth::CookieFile(path.as_ref().to_path_buf())
    }

    /// Build the `Authorization` header value, reading the cookie file if needed.
    #[cfg(any(feature = "sync", feature = "aio"))]
    pub(crate) fn header_value(&self) -> Result<Option<String>> {
        match self {
            Auth::None => Ok(None),
            Auth::UserPass(u, p) => Ok(Some(format!(
                "Basic {}",
                base64(format!("{u}:{p}").as_bytes())
            ))),
            Auth::CookieFile(path) => {
                let raw = std::fs::read_to_string(path).map_err(|e| {
                    Error::Config(format!("cannot read cookie file {}: {e}", path.display()))
                })?;
                let creds = raw.trim_end_matches(['\n', '\r']);
                if !creds.contains(':') {
                    return Err(Error::Config(format!(
                        "cookie file {} does not contain `user:password`",
                        path.display()
                    )));
                }
                Ok(Some(format!("Basic {}", base64(creds.as_bytes()))))
            }
        }
    }
}

/// Minimal standard base64 encoder, so the crate needs no base64 dependency.
#[cfg(any(feature = "sync", feature = "aio"))]
fn base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(any(feature = "sync", feature = "aio"))]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64(b"user:pass"), "dXNlcjpwYXNz");
    }

    #[test]
    #[cfg(any(feature = "sync", feature = "aio"))]
    fn none_yields_no_header() {
        assert_eq!(Auth::None.header_value().unwrap(), None);
    }

    #[test]
    #[cfg(any(feature = "sync", feature = "aio"))]
    fn user_pass_yields_basic_header() {
        let a = Auth::user_pass("user", "pass");
        assert_eq!(a.header_value().unwrap().unwrap(), "Basic dXNlcjpwYXNz");
    }

    #[test]
    #[cfg(any(feature = "sync", feature = "aio"))]
    fn cookie_file_is_read_and_encoded() {
        let dir = std::env::temp_dir().join("bitcoin_rpc_auth_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".cookie");
        // Core writes `__cookie__:<random>` with no newline; tolerate one anyway.
        std::fs::write(&path, "user:pass\n").unwrap();
        let a = Auth::cookie_file(&path);
        assert_eq!(a.header_value().unwrap().unwrap(), "Basic dXNlcjpwYXNz");
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    #[cfg(any(feature = "sync", feature = "aio"))]
    fn missing_cookie_file_is_a_config_error() {
        let a = Auth::cookie_file("/nonexistent/.cookie");
        match a.header_value() {
            Err(Error::Config(m)) => assert!(m.contains("cookie")),
            other => panic!("expected Config error, got {other:?}"),
        }
    }
}
