//! Internal `serde` helpers, not part of the public API.

use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError};
use serde_json::Value;

/// Deserialize a JSON string or an array of strings into `Vec<String>`.
///
/// Bitcoin Core normally emits `warnings` as an array, but a node started with
/// `-deprecatedrpc=warnings` emits a bare string instead. A bare string becomes
/// a one-element vector so the public type can stay `Vec<String>` regardless of
/// which wire form the node uses.
pub fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
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
}
