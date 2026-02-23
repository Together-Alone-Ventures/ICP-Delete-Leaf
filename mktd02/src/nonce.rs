//! # Monotonic Nonce Management
//!
//! Load from stable memory, increment-and-persist, never reuse.
//! The nonce is incremented *before* use in hash computations,
//! ensuring that if the message traps after increment, the nonce
//! value is "burned" (never reused) due to stable memory rollback.

use crate::storage::{with_storage_mut, StorableU64};

/// Increment the nonce and return the new value.
///
/// The nonce is persisted to stable memory immediately.
/// If the enclosing message traps, stable memory rolls back,
/// so the nonce is effectively "burned" -- never reused.
pub(crate) fn increment_nonce() -> u64 {
    with_storage_mut(|s| {
        let current = s.nonce.get().0;
        let next = current + 1;
        s.nonce
            .set(StorableU64(next))
            .expect("MKTd02: failed to persist nonce");
        next
    })
}

/// Read the current nonce value without incrementing.
pub(crate) fn current_nonce() -> u64 {
    crate::storage::with_storage(|s| s.nonce.get().0)
}
