//! # Deletion Engine
//!
//! Domain tags: `MKTD02_TOMBSTONE_HASH_V1`, `MKTD02_EVENT_V1`
//!
//! Core deletion flow is synchronous within a single message.
//!
//! ## v0.2.0 Changes
//!
//! - Phase 1: `manifest_hash` removed from `deletion_event_hash` preimage.
//!   `commit_mode` removed from receipt. New fields added.
//! - Phase 2: After publishing certified commitment, the finalization
//!   lock is acquired. This prevents any code path from changing
//!   certified data until the receipt is finalized via
//!   `mktd_finalize_receipt()`.

use crate::certified::publish_certified_commitment;
use crate::nonce::increment_nonce;
use crate::state::compute_state_hash;
use crate::storage::{
    with_storage, with_storage_mut, Hash32, MetaCell, OptionalTimestamp, ReceiptBytes,
};
use crate::trait_def::MKTdDataSource;
use crate::MktdConfig;
use zombie_core::hashing::{hash_with_tag, TAG_EVENT, TAG_TOMBSTONE_HASH, ZERO_HASH};
use zombie_core::receipt::{compute_receipt_id, DeletionReceipt, ProtocolVersion};
use zombie_core::tombstone::tombstone_constant;

#[derive(Debug, Clone)]
pub enum DeletionError {
    AlreadyTombstoned,
    NotInitialised,
}

impl core::fmt::Display for DeletionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AlreadyTombstoned => write!(f, "MKTd02: canister is already tombstoned"),
            Self::NotInitialised => write!(f, "MKTd02: not initialised"),
        }
    }
}

/// Execute the full deletion flow (Phase A). Returns the receipt_id on success.
///
/// The receipt is created with `bls_certificate: None` and
/// `trust_root_key: vec![]`. These fields are populated during
/// finalization (Phase C — see `finalize_receipt`).
///
/// After this call succeeds, the **finalization lock is held**. No
/// code path may change certified data until `finalize_receipt()` is
/// called. This is a hard invariant enforced in `publish_certified_commitment`.
pub fn execute_deletion<A: MKTdDataSource>(
    adapter: &mut A,
    config: &MktdConfig,
) -> Result<[u8; 32], DeletionError> {
    // (a) Validate not tombstoned
    if crate::guard::is_tombstoned() {
        return Err(DeletionError::AlreadyTombstoned);
    }
    if !crate::guard::is_initialised() {
        return Err(DeletionError::NotInitialised);
    }

    let canister_id = ic_cdk::id();
    let timestamp = ic_cdk::api::time();

    // (b) Capture pre_state_hash
    let pre_state_hash = compute_state_hash(&adapter.get_state_bytes());

    // (c) Tombstone
    adapter.tombstone_state();

    // (c2) Post-tombstone invariant check
    if !adapter.is_tombstoned() {
        ic_cdk::trap("MKTd02: post-tombstone invariant failed — adapter.is_tombstoned() returned false after tombstone_state()");
    }

    // (d) Capture post_state_hash
    let post_state_hash = compute_state_hash(&adapter.get_state_bytes());

    // (e) Increment nonce
    let nonce = increment_nonce();

    // (f) Compute tombstone_hash
    let tombstone_hash = hash_with_tag(TAG_TOMBSTONE_HASH, &[
        canister_id.as_slice(),
        tombstone_constant(),
        &timestamp.to_be_bytes(),
        &nonce.to_be_bytes(),
    ]);

    // (g) Read module_hash from storage
    let module_hash = with_storage(|s| s.meta.get().module_hash);

    // (h) Compute deletion_event_hash
    //     v0.2.0: manifest_hash removed from preimage
    let deletion_event_hash = hash_with_tag(TAG_EVENT, &[
        &pre_state_hash,
        &post_state_hash,
        &timestamp.to_be_bytes(),
        &module_hash,
        &nonce.to_be_bytes(),
    ]);

    // (i) Store tombstoned_at (engine-owned; see storage.rs docs)
    with_storage_mut(|s| {
        s.tombstoned_at
            .set(OptionalTimestamp(Some(timestamp)))
            .expect("MKTd02: failed to store tombstoned_at");
        s.deletion_event_hash
            .set(Hash32(deletion_event_hash))
            .expect("MKTd02: failed to store deletion_event_hash");
        s.state_hash
            .set(Hash32(post_state_hash))
            .expect("MKTd02: failed to store post_state_hash");
    });

    // (j) Compute + publish certified_commitment
    //     Note: finalization lock is NOT yet held, so this call succeeds.
    let certified_commitment =
        publish_certified_commitment(&post_state_hash, &deletion_event_hash);

    // (j2) Acquire finalization lock — from this point, no code path
    //      may call certified_data_set() until finalize_receipt() releases it.
    crate::storage::acquire_finalization_lock();

    // (k) Compute receipt_id and construct receipt
    let receipt_id = compute_receipt_id(&canister_id, nonce);

    let receipt = DeletionReceipt {
        protocol_version: ProtocolVersion::V2.into(),
        receipt_id,
        canister_id,
        subnet_id: config.subnet_id,
        pre_state_hash,
        post_state_hash,
        tombstone_hash,
        deletion_event_hash,
        certified_commitment,
        module_hash,
        timestamp,
        nonce,
        bls_certificate: None,      // Populated during finalization (Phase C)
        trust_root_key: vec![],      // Populated during finalization (Phase C)
    };

    // (l) Store receipt as CBOR in StableBTreeMap
    let mut cbor_buf = Vec::new();
    ciborium::into_writer(&receipt, &mut cbor_buf)
        .expect("MKTd02: failed to CBOR-encode receipt");
    with_storage_mut(|s| {
        s.receipts
            .insert(Hash32(receipt_id), ReceiptBytes(cbor_buf));
    });

    // (m) Return receipt_id
    Ok(receipt_id)
}

/// Upgrade cascade: recompute state hash and update module_hash.
///
/// v0.2.0: Always recomputes state_hash and republishes certified_commitment.
/// If the finalization lock is held (receipt pending), the call to
/// `publish_certified_commitment` will trap — this is intentional.
/// You must finalize the pending receipt before upgrading.
pub(crate) fn upgrade_cascade<A: MKTdDataSource>(
    adapter: &A,
    module_hash: [u8; 32],
) {
    // Always recompute state_hash (defensive — catches adapter changes)
    let state_bytes = adapter.get_state_bytes();
    let new_state_hash = compute_state_hash(&state_bytes);
    with_storage_mut(|s| {
        s.state_hash
            .set(Hash32(new_state_hash))
            .expect("MKTd02: failed to update state_hash in cascade");
    });

    // Always republish certified_commitment
    // (Will trap if finalization lock is held — see certified.rs)
    let existing_event_hash = with_storage(|s| s.deletion_event_hash.get().0);
    publish_certified_commitment(&new_state_hash, &existing_event_hash);

    // Update module_hash unconditionally
    with_storage_mut(|s| {
        let mut meta = s.meta.get().clone();
        meta.module_hash = module_hash;
        s.meta
            .set(meta)
            .expect("MKTd02: failed to update module_hash in meta cell");
    });
}

/// First-time initialisation logic.
pub(crate) fn first_init<A: MKTdDataSource>(
    adapter: &A,
    config: &MktdConfig,
    module_hash: [u8; 32],
) {
    let timestamp = ic_cdk::api::time();

    crate::state::init_state_hash(&adapter.get_state_bytes());

    let state_hash = crate::state::read_state_hash();
    publish_certified_commitment(&state_hash, &ZERO_HASH);

    with_storage_mut(|s| {
        let meta = MetaCell {
            schema_version: crate::storage::schema_version(),
            memory_base: config.base_memory_id as u32,
            initialised_at: Some(timestamp),
            module_hash,
        };
        s.meta
            .set(meta)
            .expect("MKTd02: failed to store meta cell");
    });
}
