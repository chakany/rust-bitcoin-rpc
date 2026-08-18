//! Helpers for building JSON-RPC parameter lists.

use serde_json::Value;

/// Build a positional parameter array, dropping trailing `null` entries.
///
/// Bitcoin Core treats an absent trailing argument and an explicit `null`
/// differently for some methods, so omitted optional arguments must really be
/// absent from the request.
pub(crate) fn positional(mut args: Vec<Value>) -> Value {
    while matches!(args.last(), Some(Value::Null)) {
        args.pop();
    }
    Value::Array(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keeps_all_present_args() {
        assert_eq!(positional(vec![json!(1), json!("a")]), json!([1, "a"]));
    }

    #[test]
    fn trims_trailing_nulls() {
        assert_eq!(positional(vec![json!(1), json!(null), json!(null)]), json!([1]));
    }

    #[test]
    fn keeps_interior_nulls() {
        assert_eq!(positional(vec![json!(1), json!(null), json!(3)]), json!([1, null, 3]));
    }

    #[test]
    fn all_nulls_becomes_empty_array() {
        assert_eq!(positional(vec![json!(null)]), json!([]));
    }
}
