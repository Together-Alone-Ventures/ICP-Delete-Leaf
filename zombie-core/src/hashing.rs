//! # Hashing Primitives & Domain Separation
//!
//! SHA-256 wrapper, domain separation tags, and byte concatenation helpers.
//!
//! ## Naming Convention Table
//!
//! | Name                       | Kind           | Purpose                                      | Used in          |
//! |----------------------------|----------------|----------------------------------------------|------------------|
//! | MKTD_TOMBSTONE_V1          | Constant seed  | Seed for TOMBSTONE_CONSTANT (bytes written)   | tombstone.rs     |
//! | MKTD02_TOMBSTONE_HASH_V1   | Domain tag     | Tag for tombstone_hash in receipt             | engine.rs        |
//! | MKTD02_EVENT_V1            | Domain tag     | Tag for deletion_event_hash                   | engine.rs        |
//! | MKTD02_CERTIFIED_V1        | Domain tag     | Tag for certified_commitment                  | certified.rs     |
//! | MKTD02_RECEIPT_V1          | Domain tag     | Tag for receipt_id derivation                 | receipt.rs       |
//! | MKTD02_SALT_V1             | Domain tag     | Tag for per-canister salt derivation          | state.rs         |
//! | MKTD02_MANIFEST_V1         | Domain tag     | Tag for manifest_hash computation             | manifest.rs      |
//!
//! **Key distinction:** The tombstone constant is a *value written to storage*;
//! domain tags are *prefixes for hash computations*. They must never be confused.

use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Constant seed (used to derive a stored value, NOT a hash prefix)
// ---------------------------------------------------------------------------

/// Seed string for the tombstone constant. The actual constant is
/// SHA-256(TOMBSTONE_SEED) -- see tombstone module.
pub const TOMBSTONE_SEED: &[u8] = b"MKTD_TOMBSTONE_V1";

// ---------------------------------------------------------------------------
// Domain separation tags (used as prefixes in hash computations)
// ---------------------------------------------------------------------------

/// Domain tag for tombstone_hash field in the deletion receipt.
pub const TAG_TOMBSTONE_HASH: &[u8] = b"MKTD02_TOMBSTONE_HASH_V1";

/// Domain tag for deletion_event_hash.
pub const TAG_EVENT: &[u8] = b"MKTD02_EVENT_V1";

/// Domain tag for certified_commitment.
pub const TAG_CERTIFIED: &[u8] = b"MKTD02_CERTIFIED_V1";

/// Domain tag for receipt_id derivation.
pub const TAG_RECEIPT: &[u8] = b"MKTD02_RECEIPT_V1";

/// Domain tag for per-canister salt derivation.
pub const TAG_SALT: &[u8] = b"MKTD02_SALT_V1";

/// Domain tag for manifest_hash computation.
pub const TAG_MANIFEST: &[u8] = b"MKTD02_MANIFEST_V1";

// ---------------------------------------------------------------------------
// SHA-256 wrapper
// ---------------------------------------------------------------------------

/// Compute SHA-256 of a single byte slice.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute SHA-256 of multiple byte slices concatenated in order.
///
/// The first slice should always be the domain separation tag.
pub fn sha256_concat(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

/// A zero-filled 32-byte hash, used as the initial value for
/// deletion_event_hash before any deletion has occurred.
pub const ZERO_HASH: [u8; 32] = [0u8; 32];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_deterministic() {
        let a = sha256(b"hello");
        let b = sha256(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn sha256_different_inputs_differ() {
        let a = sha256(b"hello");
        let b = sha256(b"world");
        assert_ne!(a, b);
    }

    #[test]
    fn sha256_known_vector() {
        let empty = sha256(b"");
        assert_eq!(
            hex::encode(empty),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_concat_matches_manual() {
        let manual = sha256(&[TAG_EVENT, b"data"].concat());
        let via_concat = sha256_concat(&[TAG_EVENT, b"data"]);
        assert_eq!(manual, via_concat);
    }

    #[test]
    fn sha256_concat_order_matters() {
        let ab = sha256_concat(&[b"a", b"b"]);
        let ba = sha256_concat(&[b"b", b"a"]);
        assert_ne!(ab, ba);
    }

    #[test]
    fn domain_tags_are_distinct() {
        let tags: &[&[u8]] = &[
            TAG_TOMBSTONE_HASH,
            TAG_EVENT,
            TAG_CERTIFIED,
            TAG_RECEIPT,
            TAG_SALT,
            TAG_MANIFEST,
            TOMBSTONE_SEED,
        ];
        for (i, a) in tags.iter().enumerate() {
            for (j, b) in tags.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "tags at index {} and {} collide", i, j);
                }
            }
        }
    }

    #[test]
    fn zero_hash_is_zero() {
        assert_eq!(ZERO_HASH, [0u8; 32]);
    }
}
