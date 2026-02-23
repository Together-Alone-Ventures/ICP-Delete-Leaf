//! # Monotonic Nonce Management
//!
//! Load from stable memory, increment-and-persist, never reuse.
//!
//! ## Trap/Rollback Semantics
//!
//! Nonce increments are atomic with the enclosing update message.
//! If the message traps after the nonce increment, stable memory
//! mutations roll back, so the nonce is **not** advanced. This means
//! the nonce value is not "burned" on trap -- it remains available
//! for the next successful message. This is safe because a trap also
//! aborts receipt emission: no receipt is produced with the rolled-back
//! nonce value.

use crate::storage::{with_storage_mut, StorableU64};

/// Increment the nonce and return the new value.
///
/// The nonce is persisted to stable memory immediately. If the
/// enclosing message traps, stable memory rolls back and the
/// increment does not persist (see module docs on trap semantics).
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
