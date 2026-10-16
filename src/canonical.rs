//! Canonical serialization: every byte sequence `crisk` hashes or writes as JSON is the RFC 8785
//! (JSON Canonicalization Scheme) serialization of the value, as in AI Change Provenance §8.2.

use anyhow::Result;
use sha2::{Digest, Sha256};

/// The RFC 8785 serialization of `value`.
pub fn jcs_bytes<T: serde::Serialize>(value: &T) -> Result<String> {
    Ok(serde_json_canonicalizer::to_string(value)?)
}

/// `sha256:<hex>` over [`jcs_bytes`].
pub fn digest<T: serde::Serialize>(value: &T) -> Result<String> {
    Ok(sha256(jcs_bytes(value)?.as_bytes()))
}

/// `sha256:<hex>` over raw bytes.
pub fn sha256(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(71);
    out.push_str("sha256:");
    for b in digest {
        write!(out, "{b:02x}").expect("writing to a String cannot fail");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jcs_sorts_keys_and_is_compact() {
        let v = serde_json::json!({"b": 1, "a": [true, null]});
        assert_eq!(jcs_bytes(&v).unwrap(), r#"{"a":[true,null],"b":1}"#);
        assert!(digest(&v).unwrap().starts_with("sha256:"));
    }
}
