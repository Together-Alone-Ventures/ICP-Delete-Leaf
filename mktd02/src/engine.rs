//! # Deletion Engine
//!
//! Domain tags: `MKTD02_TOMBSTONE_HASH_V1`, `MKTD02_EVENT_V1`
//!
//! Core deletion flow is synchronous within a single message.

use crate::certified::publish_certified_commitment;
use crate::nonce::increment_nonce;
use crate::state::compute_state_hash;
use crate::storage::{
    with_storage, with_storage_mut, Hash32, MetaCell, OptionalTimestamp, ReceiptBytes,
};
use crate::trait_def::MKTdDataSource;
use crate::MktdConfig;
use candid::Principal;
use zombie_core::hashing::{sha256_concat, TAG_EVENT, TAG_TOMBSTONE_HASH, ZERO_HASH};
use zombie_core::receipt::{compute_receipt_id, DeletionReceipt};
use zombie_core::tombstone::tombstone_constant;

/// Errors from execute_deletion().
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

/// Execute the full deletion flow. Returns the receipt_id on success.
///
/// ## Flow (synchronous, single message)
/// (a) Validate not tombstoned
/// (b) Capture pre_state_hash
/// (c) Call adapter.tombstone_state()
/// (c2) Post-tombstone invariant check
/// (d) Capture post_state_hash
/// (e) Increment nonce
/// (f) Compute tombstone_hash
/// (g) Read module_hash from meta cell
/// (h) Compute deletion_event_hash
/// (i) Store tombstoned_at
/// (j) Compute + publish certified_commitment
/// (k) Construct receipt
/// (l) Store receipt
/// (m) Return receipt_id
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
    let tombstone_hash = sha256_concat(&[
        TAG_TOMBSTONE_HASH,
        canister_id.as_slice(),
        tombstone_constant(),
        &timestamp.to_be_bytes(),
        &nonce.to_be_bytes(),
    ]);

    // (g) Read module_hash and manifest_hash from storage
    let (module_hash, manifest_hash) = with_storage(|s| {
        (s.meta.get().module_hash, s.manifest_hash.get().0)
    });

    // (h) Compute deletion_event_hash
    let deletion_event_hash = sha256_concat(&[
        TAG_EVENT,
        &pre_state_hash,
        &post_state_hash,
        &timestamp.to_be_bytes(),
        &module_hash,
        &manifest_hash,
        &nonce.to_be_bytes(),
    ]);

    // (i) Store tombstoned_at
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
    let certified_commitment =
        publish_certified_commitment(&post_state_hash, &deletion_event_hash);

    // (k) Compute receipt_id and construct receipt
    let receipt_id = compute_receipt_id(&canister_id, nonce);

    let receipt = DeletionReceipt {
        receipt_id,
        canister_id,
        subnet_id: config.subnet_id,
        commit_mode: adapter.mode().as_str().to_string(),
        pre_state_hash,
        post_state_hash,
        tombstone_hash,
        deletion_event_hash,
        certified_commitment,
        manifest_hash,
        module_hash,
        timestamp,
        nonce,
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

/// Upgrade cascade: detect manifest changes, update module_hash.
///
/// Called from on_post_upgrade().
pub(crate) fn upgrade_cascade<A: MKTdDataSource>(
    adapter: &A,
    module_hash: [u8; 32],
) {
    let new_manifest_hash = adapter.manifest_hash();

    // (1-2) Check for manifest change
    let stored_manifest_hash = with_storage(|s| s.manifest_hash.get().0);

    if new_manifest_hash != stored_manifest_hash {
        // Manifest changed: full recomputation cascade
        with_storage_mut(|s| {
            s.manifest_hash
                .set(Hash32(new_manifest_hash))
                .expect("MKTd02: failed to update manifest_hash");
        });

        // Recompute state_hash under new manifest
        let state_bytes = adapter.get_state_bytes();
        let new_state_hash = compute_state_hash(&state_bytes);
        with_storage_mut(|s| {
            s.state_hash
                .set(Hash32(new_state_hash))
                .expect("MKTd02: failed to update state_hash in cascade");
        });

        // Recompute certified_commitment with existing deletion_event_hash
        let existing_event_hash = with_storage(|s| s.deletion_event_hash.get().0);
        publish_certified_commitment(&new_state_hash, &existing_event_hash);
    }

    // (3) Update module_hash unconditionally
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
) {
    let timestamp = ic_cdk::api::time();

    // Compute and store manifest_hash
    let manifest_hash = adapter.manifest_hash();
    with_storage_mut(|s| {
        s.manifest_hash
            .set(Hash32(manifest_hash))
            .expect("MKTd02: failed to store manifest_hash");
    });

    // Compute and store initial state_hash
    crate::state::init_state_hash(&adapter.get_state_bytes());

    // Compute and publish initial certified_commitment
    let state_hash = crate::state::read_state_hash();
    publish_certified_commitment(&state_hash, &ZERO_HASH);

    // Store meta cell
    with_storage_mut(|s| {
        let meta = MetaCell {
            schema_version: crate::storage::schema_version(),
            memory_base: config.base_memory_id as u32,
            initialised_at: Some(timestamp),
            module_hash: [0u8; 32], // zeros at init (chicken-and-egg)
        };
        s.meta
            .set(meta)
            .expect("MKTd02: failed to store meta cell");
    });
}
