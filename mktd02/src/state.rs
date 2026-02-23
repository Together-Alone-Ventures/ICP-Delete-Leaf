//! # State Hash Computation
//!
//! Domain tag: `MKTD02_SALT_V1`
//!
//! The per-canister salt is derived at runtime (not stored):
//! `mktd_salt = SHA-256(MKTD02_SALT_V1 || canister_id_bytes)`
//!
//! State hash: `SHA-256(mktd_salt || state_bytes)`

use crate::storage::{with_storage, with_storage_mut, Hash32};
use zombie_core::hashing::{sha256_concat, TAG_SALT};

/// Derive the per-canister salt. Deterministic from canister_id.
pub(crate) fn compute_salt() -> [u8; 32] {
    sha256_concat(&[TAG_SALT, ic_cdk::id().as_slice()])
}

/// Compute state_hash from PII state bytes.
///
/// `state_hash = SHA-256(mktd_salt || state_bytes)`
pub(crate) fn compute_state_hash(state_bytes: &[u8]) -> [u8; 32] {
    let salt = compute_salt();
    sha256_concat(&[&salt, state_bytes])
}

/// Compute and store the initial state hash. Called during init().
pub(crate) fn init_state_hash(state_bytes: &[u8]) {
    let hash = compute_state_hash(state_bytes);
    with_storage_mut(|s| {
        s.state_hash
            .set(Hash32(hash))
            .expect("MKTd02: failed to store initial state_hash");
    });
}

/// Recompute and store state hash after a PII mutation.
/// Called by the host canister after each write to PII fields.
pub(crate) fn refresh_state_hash_internal(state_bytes: &[u8]) {
    let hash = compute_state_hash(state_bytes);
    with_storage_mut(|s| {
        s.state_hash
            .set(Hash32(hash))
            .expect("MKTd02: failed to update state_hash");
    });
}

/// Read the current state hash.
pub(crate) fn read_state_hash() -> [u8; 32] {
    with_storage(|s| s.state_hash.get().0)
}
