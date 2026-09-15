//! Shared client configuration.

use std::time::Duration;

use crate::{Auth, Error, Result};

/// The default cap on a reply body, 64 MiB.
///
/// Generous for every response this crate types (a verbosity-3 `getblock`
/// of a full 4 MB block is on the order of 20-30 MB of JSON) while still
/// bounding what a misbehaving node or proxy can make the client allocate.
pub(crate) const DEFAULT_MAX_RESPONSE_SIZE: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub url: String,
    pub auth: Auth,
    pub timeout: Option<Duration>,
    pub max_response_size: Option<usize>,
}

impl Config {
    pub(crate) fn new(url: impl Into<String>) -> Self {
        Config {
            url: url.into(),
            auth: Auth::None,
            timeout: Some(Duration::from_secs(30)),
            max_response_size: Some(DEFAULT_MAX_RESPONSE_SIZE),
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if !(self.url.starts_with("http://") || self.url.starts_with("https://")) {
            return Err(Error::Config(format!(
                "url must start with http:// or https://, got `{}`",
                self.url
            )));
        }
        if self.max_response_size == Some(0) {
            return Err(Error::Config(
                "max_response_size must be greater than zero (use None for no limit)".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_no_auth_and_30s_timeout() {
        let c = Config::new("http://127.0.0.1:8332");
        assert_eq!(c.auth, crate::Auth::None);
        assert_eq!(c.timeout, Some(std::time::Duration::from_secs(30)));
        assert_eq!(c.max_response_size, Some(64 * 1024 * 1024));
    }

    #[test]
    fn rejects_a_zero_response_size_limit() {
        let mut c = Config::new("http://127.0.0.1:8332");
        c.max_response_size = Some(0);
        assert!(matches!(c.validate(), Err(crate::Error::Config(_))));
        c.max_response_size = None;
        assert!(c.validate().is_ok());
    }

    #[test]
    fn rejects_non_http_url() {
        let c = Config::new("127.0.0.1:8332");
        assert!(matches!(c.validate(), Err(crate::Error::Config(_))));
    }

    #[test]
    fn accepts_http_and_https() {
        assert!(Config::new("http://127.0.0.1:8332").validate().is_ok());
        assert!(Config::new("https://node.example:8332").validate().is_ok());
    }
}
