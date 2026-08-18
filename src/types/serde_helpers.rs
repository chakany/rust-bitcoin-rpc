//! Internal `serde` helpers, not part of the public API.

use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError};
use serde_json::Value;

/// Deserialize a JSON string or an array of strings into `Vec<String>`.
///
/// Bitcoin Core normally emits `warnings` as an array, but a node started with
/// `-deprecatedrpc=warnings` emits a bare string instead: the most recent
/// warning, or `""` when there is none (`node::GetWarningsForRpc`,
/// `src/node/warnings.cpp:56-58`). A non-empty string becomes a one-element
/// vector and `""` becomes an empty one, so the public type can stay
/// `Vec<String>` and `warnings.is_empty()` answers the same question on either
/// wire form.
pub fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        // The legacy form says "no warnings" with `""`, not with one empty warning.
        Value::String(s) if s.is_empty() => Ok(Vec::new()),
        Value::String(s) => Ok(vec![s]),
        value @ Value::Array(_) => serde_json::from_value(value).map_err(DeError::custom),
        other => Err(DeError::custom(format!(
            "expected a string or an array of strings, got {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[derive(serde::Deserialize)]
    struct Wrapper {
        #[serde(deserialize_with = "super::string_or_seq_string")]
        warnings: Vec<String>,
    }

    #[test]
    fn accepts_array_form() {
        let w: Wrapper = serde_json::from_value(json!({"warnings": ["a", "b"]})).unwrap();
        assert_eq!(w.warnings, vec!["a", "b"]);
    }

    #[test]
    fn accepts_legacy_string_form() {
        let w: Wrapper = serde_json::from_value(json!({"warnings": "single warning"})).unwrap();
        assert_eq!(w.warnings, vec!["single warning"]);
    }

    #[test]
    fn legacy_empty_string_means_no_warnings() {
        let w: Wrapper = serde_json::from_value(json!({"warnings": ""})).unwrap();
        assert!(w.warnings.is_empty(), "got {:?}", w.warnings);
    }
}
