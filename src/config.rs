//! Shared client configuration.

use std::time::Duration;

use crate::{Auth, Error, Result};

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub url: String,
    pub auth: Auth,
    pub timeout: Duration,
}

impl Config {
    pub(crate) fn new(url: impl Into<String>) -> Self {
        Config { url: url.into(), auth: Auth::None, timeout: Duration::from_secs(30) }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.url.starts_with("http://") || self.url.starts_with("https://") {
            Ok(())
        } else {
            Err(Error::Config(format!("url must start with http:// or https://, got `{}`", self.url)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_no_auth_and_30s_timeout() {
        let c = Config::new("http://127.0.0.1:8332");
        assert_eq!(c.auth, crate::Auth::None);
        assert_eq!(c.timeout, std::time::Duration::from_secs(30));
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
