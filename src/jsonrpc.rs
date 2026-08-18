//! JSON-RPC 2.0 request and response envelopes.
//!
//! v31.1 answers 2.0 requests with HTTP 200 even for RPC errors, putting the
//! error in the `error` member, so callers have a single error path.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{Error, Result, RpcError};

#[derive(Debug, Serialize)]
pub(crate) struct Request<'a> {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: &'a str,
    pub params: Value,
}

impl<'a> Request<'a> {
    pub(crate) fn new(id: u64, method: &'a str, params: Value) -> Self {
        Request { jsonrpc: "2.0", id, method, params }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Response {
    // `deserialize_with` (not plain `Option<Value>`) is needed because serde
    // collapses an explicit JSON `null` and an absent field to the same
    // `None`, and `into_result` must tell them apart.
    #[serde(default, deserialize_with = "some_if_present")]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<ErrorBody>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ErrorBody {
    pub code: i32,
    pub message: String,
}

// Wraps a present field's value in `Some`, even when that value is `null`,
// so `#[serde(default)]` can supply `None` only when the field is absent.
fn some_if_present<'de, D>(deserializer: D) -> std::result::Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    Value::deserialize(deserializer).map(Some)
}

impl Response {
    pub(crate) fn into_result(self) -> Result<Value> {
        if let Some(e) = self.error {
            return Err(Error::Rpc(RpcError { code: e.code, message: e.message }));
        }
        self.result
            .ok_or_else(|| Error::Transport("reply contained neither result nor error".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_serializes_as_jsonrpc_2_0() {
        let req = Request { jsonrpc: "2.0", id: 7, method: "getblockhash", params: json!([100]) };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v, json!({"jsonrpc":"2.0","id":7,"method":"getblockhash","params":[100]}));
    }

    #[test]
    fn success_response_yields_result() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"blocks":42}}"#;
        let resp: Response = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.into_result().unwrap(), json!({"blocks":42}));
    }

    #[test]
    fn error_response_yields_rpc_error() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-8,"message":"Block not found"}}"#;
        let resp: Response = serde_json::from_str(raw).unwrap();
        match resp.into_result() {
            Err(crate::Error::Rpc(e)) => {
                assert_eq!(e.code, -8);
                assert_eq!(e.message, "Block not found");
            }
            other => panic!("expected Rpc error, got {other:?}"),
        }
    }

    #[test]
    fn null_result_is_ok_null() {
        // `stop` and `waitfornewblock` legitimately return null-ish payloads.
        let raw = r#"{"jsonrpc":"2.0","id":1,"result":null}"#;
        let resp: Response = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.into_result().unwrap(), json!(null));
    }

    #[test]
    fn response_without_result_or_error_is_a_transport_error() {
        let raw = r#"{"jsonrpc":"2.0","id":1}"#;
        let resp: Response = serde_json::from_str(raw).unwrap();
        assert!(matches!(resp.into_result(), Err(crate::Error::Transport(_))));
    }
}
