//! Round-trips real `decodepsbt` payloads through [`PsbtDecoded`].
//!
//! Deserializing alone cannot catch a dropped field: every per-input and
//! per-output field is optional, so a misspelled `serde(rename)` silently
//! yields `None` and the test still passes. Re-serializing and comparing
//! against the original payload does catch it -- the field goes missing from
//! the output.
//!
//! Nulls are stripped from the re-serialized side first. The pre-existing
//! `Transaction` type marks its optional fields `serde(default)` without
//! `skip_serializing_if`, so fields a PSBT's embedded transaction never has
//! (`confirmations`, `blockhash`, `fee`, ...) come back as explicit nulls.
//! That is a property of the existing type, not a dropped field.
#![cfg(feature = "serde")]
use bitcoin_rpc::types::*;

fn check(name: &str, raw: &str) {
    let original: serde_json::Value = serde_json::from_str(raw).unwrap();
    let decoded: PsbtDecoded = serde_json::from_str(raw).unwrap();
    let mut back = serde_json::to_value(&decoded).unwrap();
    strip_nulls(&mut back);
    if back != original {
        // Report the first differing path rather than dumping both blobs.
        let mut diffs = Vec::new();
        diff("", &original, &back, &mut diffs);
        panic!(
            "{name}: {} difference(s):\n{}",
            diffs.len(),
            diffs.join("\n")
        );
    }
}

/// Drops every null-valued object member, recursively.
fn strip_nulls(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            map.retain(|_, value| !value.is_null());
            for value in map.values_mut() {
                strip_nulls(value);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                strip_nulls(item);
            }
        }
        _ => {}
    }
}

fn diff(path: &str, a: &serde_json::Value, b: &serde_json::Value, out: &mut Vec<String>) {
    use serde_json::Value::*;
    match (a, b) {
        (Object(x), Object(y)) => {
            for (k, v) in x {
                match y.get(k) {
                    Some(w) => diff(&format!("{path}.{k}"), v, w, out),
                    None => out.push(format!("  MISSING {path}.{k} = {v}")),
                }
            }
            for k in y.keys() {
                if !x.contains_key(k) {
                    out.push(format!("  EXTRA   {path}.{k}"));
                }
            }
        }
        (Array(x), Array(y)) => {
            if x.len() != y.len() {
                out.push(format!("  LEN     {path}: {} vs {}", x.len(), y.len()));
            }
            for (i, (v, w)) in x.iter().zip(y).enumerate() {
                diff(&format!("{path}[{i}]"), v, w, out);
            }
        }
        _ if a != b => out.push(format!("  VALUE   {path}: {a} vs {b}")),
        _ => {}
    }
}

#[test]
fn round_trip_live_payloads() {
    check("segwit", include_str!("data/psbt_segwit.json"));
    check("combined", include_str!("data/psbt_combined.json"));
    check("finalized", include_str!("data/psbt_finalized.json"));
    check("taproot_tree", include_str!("data/psbt_taproot_tree.json"));
    check("musig2", include_str!("data/psbt_musig2.json"));
}
