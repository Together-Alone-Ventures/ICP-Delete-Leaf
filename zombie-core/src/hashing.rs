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

/// A domain separation tag. Wraps a static byte slice to enforce
/// tag-first ordering in hash computations via [`hash_with_tag`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainTag(pub &'static [u8]);

/// Domain tag for tombstone_hash field in the deletion receipt.
pub const TAG_TOMBSTONE_HASH: DomainTag = DomainTag(b"MKTD02_TOMBSTONE_HASH_V1");

/// Domain tag for deletion_event_hash.
pub const TAG_EVENT: DomainTag = DomainTag(b"MKTD02_EVENT_V1");

/// Domain tag for certified_commitment.
pub const TAG_CERTIFIED: DomainTag = DomainTag(b"MKTD02_CERTIFIED_V1");

/// Domain tag for receipt_id derivation.
pub const TAG_RECEIPT: DomainTag = DomainTag(b"MKTD02_RECEIPT_V1");

/// Domain tag for per-canister salt derivation.
pub const TAG_SALT: DomainTag = DomainTag(b"MKTD02_SALT_V1");

/// Domain tag for manifest_hash computation.
pub const TAG_MANIFEST: DomainTag = DomainTag(b"MKTD02_MANIFEST_V1");

// ---------------------------------------------------------------------------
// SHA-256 wrapper
// ---------------------------------------------------------------------------

/// Compute SHA-256 of a single byte slice.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute a domain-separated hash: `SHA-256(tag || part_0 || part_1 || ...)`.
///
/// The [`DomainTag`] newtype enforces that the tag is always the first
/// element in the hash preimage, preventing accidental misordering.
pub fn hash_with_tag(tag: DomainTag, parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(tag.0);
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

/// Compute SHA-256 of multiple byte slices concatenated in order.
///
/// **Prefer [`hash_with_tag`] for domain-separated hashes.** This
/// function is for cases without a domain tag (e.g., salt || state_bytes).
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
    fn hash_with_tag_matches_concat() {
        let via_tag = hash_with_tag(TAG_EVENT, &[b"data"]);
        let via_concat = sha256_concat(&[TAG_EVENT.0, b"data"]);
        assert_eq!(via_tag, via_concat);
    }

    #[test]
    fn hash_with_tag_order_matters() {
        let ab = hash_with_tag(TAG_EVENT, &[b"a", b"b"]);
        let ba = hash_with_tag(TAG_EVENT, &[b"b", b"a"]);
        assert_ne!(ab, ba);
    }

    #[test]
    fn domain_tags_are_distinct() {
        let tags: &[DomainTag] = &[
            TAG_TOMBSTONE_HASH,
            TAG_EVENT,
            TAG_CERTIFIED,
            TAG_RECEIPT,
            TAG_SALT,
            TAG_MANIFEST,
        ];
        for (i, a) in tags.iter().enumerate() {
            for (j, b) in tags.iter().enumerate() {
                if i != j {
                    assert_ne!(a.0, b.0, "tags at index {} and {} collide", i, j);
                }
            }
        }
    }

    #[test]
    fn tombstone_seed_differs_from_all_tags() {
        let tags: &[DomainTag] = &[
            TAG_TOMBSTONE_HASH, TAG_EVENT, TAG_CERTIFIED,
            TAG_RECEIPT, TAG_SALT, TAG_MANIFEST,
        ];
        for tag in tags {
            assert_ne!(TOMBSTONE_SEED, tag.0);
        }
    }

    #[test]
    fn zero_hash_is_zero() {
        assert_eq!(ZERO_HASH, [0u8; 32]);
    }
}
