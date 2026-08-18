//! Helpers for building JSON-RPC parameter lists.
//!
//! Public so downstream crates writing their own RPC-method extension traits
//! (the pattern this crate itself uses, see [`crate::sync::RpcCall`]) can give
//! omitted optional arguments the same semantics Bitcoin Core expects.

use serde_json::Value;

/// Build a positional parameter array, dropping trailing `null` entries.
///
/// Bitcoin Core distinguishes an *absent* trailing argument (use the method's
/// default) from an *explicit* `null` (often a hard error for that argument),
/// so an omitted optional argument must be dropped from the end of the array,
/// not sent as `null`. Only trailing `null`s are dropped; an interior `null`
/// is left in place, since Core does accept `null` there to mean "default for
/// this one argument" (e.g. `getblock(hash, null, verbosity)`).
///
/// ```
/// use bitcoin_rpc::params::positional;
/// use serde_json::json;
///
/// let (txid, vout) = ("abc123", 0u32);
/// assert_eq!(positional(vec![json!(txid), json!(vout)]), json!(["abc123", 0]));
/// ```
pub fn positional(mut args: Vec<Value>) -> Value {
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
        assert_eq!(
            positional(vec![json!(1), json!(null), json!(null)]),
            json!([1])
        );
    }

    #[test]
    fn keeps_interior_nulls() {
        assert_eq!(
            positional(vec![json!(1), json!(null), json!(3)]),
            json!([1, null, 3])
        );
    }

    #[test]
    fn all_nulls_becomes_empty_array() {
        assert_eq!(positional(vec![json!(null)]), json!([]));
    }
}
