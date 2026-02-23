//! # Deletion Engine
//!
//! Domain tags: `MKTD02_TOMBSTONE_HASH_V1`, `MKTD02_EVENT_V1`
//!
//! Core deletion flow (synchronous, single message):
//! 1. Validate pre-conditions (not tombstoned, initialised)
//! 2. Capture pre-deletion state_hash
//! 3. Call adapter.tombstone_state()
//! 4. Verify post-tombstone invariant (adapter.is_tombstoned() == true)
//! 5. Compute post-deletion state_hash
//! 6. Compute tombstone_hash, deletion_event_hash
//! 7. Compute certified_commitment, publish via set_certified_data
//! 8. Mint nonce, compute receipt_id
//! 9. Store receipt, set tombstoned_at
//! 10. Return DeletionReceipt
//!
//! Upgrade cascade (on_post_upgrade):
//! 1. Recompute manifest_hash; if changed → recompute state_hash →
//!    recompute certified_commitment → publish
//! 2. Update module_hash in meta cell unconditionally

// TODO(Phase 2.6): execute_deletion(), on_post_upgrade() cascade
