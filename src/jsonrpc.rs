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
        Request {
            jsonrpc: "2.0",
            id,
            method,
            params,
        }
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
            return Err(Error::Rpc(RpcError {
                code: e.code,
                message: e.message,
            }));
        }
        self.result
            .ok_or_else(|| Error::Transport("reply contained neither result nor error".into()))
    }
}

/// Turn an HTTP status and reply body into a JSON-RPC result.
///
/// v31.1 answers JSON-RPC 2.0 with HTTP 200 even for RPC errors, so a parseable
/// body always wins regardless of status. When the body is empty or not JSON,
/// a 2xx status is a malformed JSON-RPC reply, while anything else means the
/// failure happened below the JSON-RPC layer, where the status is the only
/// information we have.
pub(crate) fn parse_reply(status: u16, bytes: &[u8]) -> Result<Value> {
    match serde_json::from_slice::<Response>(bytes) {
        Ok(reply) => reply.into_result(),
        // A 2xx that is not JSON is a malformed reply; anything else is an HTTP failure.
        Err(e) if (200..300).contains(&status) => Err(Error::Json(e)),
        Err(_) => Err(Error::Transport(http_failure(
            status,
            &String::from_utf8_lossy(bytes),
        ))),
    }
}

fn http_failure(status: u16, body: &str) -> String {
    let hint = match status {
        401 | 403 => " — check the RPC credentials (--rpcuser/--rpcpassword or the cookie file)",
        404 => " — check the URL path",
        _ => "",
    };
    let body = body.trim();
    if body.is_empty() {
        format!("node returned HTTP {status}{hint}")
    } else {
        format!("node returned HTTP {status}{hint}: {body}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_serializes_as_jsonrpc_2_0() {
        let req = Request {
            jsonrpc: "2.0",
            id: 7,
            method: "getblockhash",
            params: json!([100]),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(
            v,
            json!({"jsonrpc":"2.0","id":7,"method":"getblockhash","params":[100]})
        );
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
        assert!(matches!(
            resp.into_result(),
            Err(crate::Error::Transport(_))
        ));
    }

    #[test]
    fn parse_reply_200_with_result_is_ok() {
        let raw = br#"{"jsonrpc":"2.0","id":1,"result":{"blocks":42}}"#;
        assert_eq!(parse_reply(200, raw).unwrap(), json!({"blocks": 42}));
    }

    #[test]
    fn parse_reply_200_with_empty_body_is_json_error() {
        // Only a non-2xx with an empty body is a transport failure; a 200
        // that answers with nothing at all is still a malformed JSON-RPC reply.
        assert!(matches!(parse_reply(200, b""), Err(Error::Json(_))));
    }

    #[test]
    fn parse_reply_200_with_error_member_is_rpc_error() {
        let raw = br#"{"jsonrpc":"2.0","id":1,"error":{"code":-8,"message":"Block not found"}}"#;
        match parse_reply(200, raw) {
            Err(Error::Rpc(e)) => {
                assert_eq!(e.code, -8);
                assert_eq!(e.message, "Block not found");
            }
            other => panic!("expected Rpc error, got {other:?}"),
        }
    }

    #[test]
    fn parse_reply_500_with_error_body_is_still_rpc_error() {
        // A 1.0-style node or a proxy may answer 500; the body still carries the error.
        let raw = br#"{"result":null,"error":{"code":-32601,"message":"Method not found"},"id":1}"#;
        match parse_reply(500, raw) {
            Err(Error::Rpc(e)) => assert_eq!(e.code, -32601),
            other => panic!("expected Rpc error, got {other:?}"),
        }
    }

    #[test]
    fn parse_reply_401_with_empty_body_is_transport_error_mentioning_credentials() {
        match parse_reply(401, b"") {
            Err(Error::Transport(m)) => {
                assert!(m.contains("401"));
                assert!(m.to_lowercase().contains("credentials"));
            }
            other => panic!("expected Transport error, got {other:?}"),
        }
    }

    #[test]
    fn parse_reply_403_with_empty_body_is_transport_error_mentioning_credentials() {
        match parse_reply(403, b"") {
            Err(Error::Transport(m)) => {
                assert!(m.contains("403"));
                assert!(m.to_lowercase().contains("credentials"));
            }
            other => panic!("expected Transport error, got {other:?}"),
        }
    }

    #[test]
    fn parse_reply_200_with_garbage_is_json_error() {
        assert!(matches!(
            parse_reply(200, b"not json at all"),
            Err(Error::Json(_))
        ));
    }

    #[test]
    fn parse_reply_502_with_html_body_is_transport_error_with_body_included() {
        let body = b"<html><body>Bad Gateway</body></html>";
        match parse_reply(502, body) {
            Err(Error::Transport(m)) => {
                assert!(m.contains("502"));
                assert!(m.contains("Bad Gateway"));
            }
            other => panic!("expected Transport error, got {other:?}"),
        }
    }
}
