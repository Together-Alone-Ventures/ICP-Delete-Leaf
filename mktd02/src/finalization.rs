//! # Receipt Finalization (Phase B + Phase C)
//!
//! Implements the two post-deletion steps that embed the BLS certificate
//! and NNS root key identifier into a pending receipt, making it
//! self-contained for offline V2 verification.
//!
//! ## Three-Phase Deletion Flow
//!
//! | Phase | Call Type | What Happens                                        |
//! |-------|-----------|-----------------------------------------------------|
//! | A     | update    | `execute_deletion()` — tombstone, emit receipt,     |
//! |       |           | publish certified data, acquire finalization lock    |
//! | B     | query     | `get_pending_certificate()` — read BLS certificate  |
//! |       |           | from IC runtime (only available in query context)    |
//! | C     | update    | `finalize_receipt()` — embed certificate + key ID   |
//! |       |           | in receipt, release finalization lock                |
//!
//! ## trust_root_key_id
//!
//! The library stamps the receipt with `zombie_core::nns_keys::active_key_id()`
//! automatically during finalization. Integrators do not supply the key ID —
//! it is determined by the build configuration (mainnet vs local-dev).
//! This eliminates the risk of an integrator supplying a wrong or fabricated key.
//!
//! ## Pending Identity Invariant
//!
//! Phase A stores the pending receipt_id in stable memory while the
//! finalization lock is held. Phase B and Phase C read this persisted value
//! directly. This avoids any recomputation coupling to mutable runtime values.

use crate::certified::read_certified_commitment;
use crate::storage::{with_storage, with_storage_mut, Hash32, ReceiptBytes};
use zombie_core::nns_keys;
use zombie_core::receipt::DeletionReceipt;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during receipt finalization.
#[derive(Debug, Clone)]
pub enum FinalizationError {
    /// No receipt is pending finalization (lock not held).
    NoPendingReceipt,
    /// The provided receipt_id does not match the pending receipt.
    ReceiptIdMismatch {
        expected: String,
        provided: String,
    },
    /// The receipt already has a BLS certificate (already finalized).
    AlreadyFinalized,
    /// The caller is not a controller of this canister.
    NotController,
    /// The receipt was not found in storage (internal error).
    ReceiptNotFound,
    /// Failed to re-encode the updated receipt.
    EncodingFailed(String),
}

impl core::fmt::Display for FinalizationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoPendingReceipt => write!(
                f, "MKTd02: no receipt pending finalization"
            ),
            Self::ReceiptIdMismatch { expected, provided } => write!(
                f, "MKTd02: receipt_id mismatch — expected {expected}, got {provided}"
            ),
            Self::AlreadyFinalized => write!(
                f, "MKTd02: receipt already finalized (bls_certificate is present)"
            ),
            Self::NotController => write!(
                f, "MKTd02: caller is not a controller — only controllers can finalize receipts"
            ),
            Self::ReceiptNotFound => write!(
                f, "MKTd02: pending receipt not found in storage (internal error)"
            ),
            Self::EncodingFailed(e) => write!(
                f, "MKTd02: failed to re-encode receipt after finalization: {e}"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Phase B: Certificate retrieval (query context)
// ---------------------------------------------------------------------------

/// Data returned by `get_pending_certificate()`.
///
/// All three fields are needed by the orchestrator to call
/// `finalize_receipt()` in Phase C.
#[derive(Debug, Clone)]
pub struct PendingCertificate {
    /// Receipt ID of the pending receipt (hex-encoded for convenience).
    pub receipt_id_hex: String,
    /// Raw receipt_id bytes (for passing to finalize_receipt).
    pub receipt_id: [u8; 32],
    /// The certified commitment bytes currently in certified data.
    pub certified_commitment: [u8; 32],
    /// The BLS certificate blob from `ic0.data_certificate()`.
    pub certificate: Vec<u8>,
}

fn read_pending_receipt_id() -> Option<[u8; 32]> {
    if !crate::storage::is_finalization_locked() {
        return None;
    }
    crate::storage::pending_receipt_id()
}

/// Phase B: Retrieve the BLS certificate for the pending receipt.
///
/// **Must be called in query context** — `ic0.data_certificate()` is
/// only available in query calls. Returns `None` if no receipt is
/// pending or if the IC runtime does not provide a certificate.
///
/// The orchestrator passes `certificate` to `finalize_receipt()` in Phase C.
/// The NNS root key ID is determined automatically by the library.
///
/// The returned `receipt_id` is loaded from persisted pending-finalization
/// state set by Phase A.
pub fn get_pending_certificate() -> Option<PendingCertificate> {
    // Source of truth: pending receipt_id persisted in Phase A.
    let receipt_id = read_pending_receipt_id()?;

    // Read certified commitment
    let certified_commitment = read_certified_commitment();

    // Get BLS certificate from IC runtime (query context only)
    let certificate = ic_cdk::api::data_certificate()?;

    Some(PendingCertificate {
        receipt_id_hex: hex::encode(receipt_id),
        receipt_id,
        certified_commitment,
        certificate,
    })
}

// ---------------------------------------------------------------------------
// Phase C: Finalize receipt (update context)
// ---------------------------------------------------------------------------

/// Phase C: Embed the BLS certificate and NNS root key ID into the pending receipt.
///
/// ## Guards
///
/// 1. **Finalization lock** — must be held (a receipt is pending)
/// 2. **Receipt ID match** — provided ID must match the pending receipt
/// 3. **Not already finalized** — `bls_certificate` must be `None`
/// 4. **Controller only** — caller must be a controller of this canister
///
/// ## Parameters
///
/// - `receipt_id`: The 32-byte receipt ID (from Phase B response)
/// - `certificate`: The BLS certificate blob (from Phase B response)
///
/// The NNS root key ID is determined automatically from the build
/// configuration via `zombie_core::nns_keys::active_key_id()`. Integrators
/// do not supply it — this prevents key ID mismatches and fabrication.
///
/// On success, the receipt's `bls_certificate` and `trust_root_key_id`
/// fields are populated and the finalization lock is released.
///
pub fn finalize_receipt(
    receipt_id: &[u8; 32],
    certificate: Vec<u8>,
) -> Result<(), FinalizationError> {
    // Guard 1: finalization lock must be held.
    //
    // Checked ahead of the controller guard to preserve the original
    // observable ordering (NoPendingReceipt is reported before NotController).
    if read_pending_receipt_id().is_none() {
        return Err(FinalizationError::NoPendingReceipt);
    }

    // Guard 2: caller must be a controller.
    // ic-cdk 0.18: `ic_cdk::caller()` → `ic_cdk::api::msg_caller()` (same ic0
    // msg_caller syscall); `is_controller` unchanged. Controller-guard behaviour
    // and the NoPendingReceipt-before-NotController ordering are preserved.
    let caller = ic_cdk::api::msg_caller();
    if !ic_cdk::api::is_controller(&caller) {
        return Err(FinalizationError::NotController);
    }

    // Remaining guards (lock / id-match / already-finalized) + embed/store/release.
    finalize_locked_receipt(receipt_id, certificate)
}

/// Phase C finalize **after host-side authorization** — performs **no
/// caller/controller check**.
///
/// # SECURITY — READ BEFORE USE
///
/// This function embeds the certificate into the pending receipt and releases
/// the finalization lock for **any** caller. It performs **no caller or
/// controller authorization** of its own. The host (the delete-pipeline
/// orchestrator) **MUST** have already completed its own authorization of the
/// deletion subject **before** invoking this function. Calling it without that
/// prior authorization lets an unauthenticated party finalize a pending receipt.
///
/// **This is crate / library API only. It MUST NOT be exposed directly as a
/// Candid `#[update]` (or any other) canister method.** Wire it only behind
/// host-side authorization that runs first.
///
/// Behaviour is otherwise identical to [`finalize_receipt`] minus the
/// controller guard: same `NoPendingReceipt` / `ReceiptIdMismatch` /
/// `AlreadyFinalized` semantics, and the finalization lock is released on success.
pub fn finalize_receipt_after_host_authorization(
    receipt_id: &[u8; 32],
    certificate: Vec<u8>,
) -> Result<(), FinalizationError> {
    finalize_locked_receipt(receipt_id, certificate)
}

/// Shared finalize core: lock / id-match / already-finalized guards, then
/// embed the certificate + NNS root key id, re-encode, store, and release the
/// finalization lock.
///
/// **Contains no caller/controller check.** Authorization is the caller's
/// responsibility: the controller check lives in [`finalize_receipt`]; the
/// host-authorization contract lives in
/// [`finalize_receipt_after_host_authorization`].
fn finalize_locked_receipt(
    receipt_id: &[u8; 32],
    certificate: Vec<u8>,
) -> Result<(), FinalizationError> {
    // Guard: finalization lock must be held
    let expected_id = match read_pending_receipt_id() {
        Some(id) => id,
        None => return Err(FinalizationError::NoPendingReceipt),
    };

    // Guard: receipt_id must match
    if receipt_id != &expected_id {
        return Err(FinalizationError::ReceiptIdMismatch {
            expected: hex::encode(expected_id),
            provided: hex::encode(receipt_id),
        });
    }

    // Load the receipt from storage
    let receipt_bytes = with_storage(|s| {
        s.receipts.get(&Hash32(*receipt_id))
    });

    let receipt_bytes = match receipt_bytes {
        Some(rb) => rb,
        None => return Err(FinalizationError::ReceiptNotFound),
    };

    let mut receipt: DeletionReceipt = ciborium::from_reader(receipt_bytes.0.as_slice())
        .map_err(|e| FinalizationError::EncodingFailed(
            format!("failed to decode pending receipt: {e}")
        ))?;

    // Guard: must not already be finalized
    if receipt.bls_certificate.is_some() {
        return Err(FinalizationError::AlreadyFinalized);
    }

    // Embed certificate and root key ID
    receipt.bls_certificate = Some(certificate);
    receipt.trust_root_key_id = nns_keys::active_key_id().to_string();

    // Re-encode and store
    let mut cbor_buf = Vec::new();
    ciborium::into_writer(&receipt, &mut cbor_buf)
        .map_err(|e| FinalizationError::EncodingFailed(e.to_string()))?;

    with_storage_mut(|s| {
        s.receipts
            .insert(Hash32(*receipt_id), ReceiptBytes(cbor_buf));
    });

    // Release finalization lock — certified data can change again
    crate::storage::release_finalization_lock();

    Ok(())
}

/// Check whether a receipt is pending finalization.
///
/// Convenience wrapper for the finalization lock check.
pub fn is_pending_finalization() -> bool {
    crate::storage::is_finalization_locked()
}

#[cfg(test)]
mod tests {
    use super::{finalize_receipt_after_host_authorization, read_pending_receipt_id, FinalizationError};
    use crate::nonce::increment_deletion_seq;
    use crate::storage::{
        acquire_finalization_lock, is_finalization_locked, pending_receipt_id,
        release_finalization_lock, set_pending_receipt_id, setup_storage, with_storage,
        with_storage_mut, Hash32, ReceiptBytes,
    };
    use candid::Principal;
    use ic_stable_structures::memory_manager::MemoryManager;
    use ic_stable_structures::DefaultMemoryImpl;
    use zombie_core::receipt::{DeletionReceipt, ProtocolVersion};

    fn setup_test_storage(base: u8) {
        let mm = MemoryManager::init(DefaultMemoryImpl::default());
        setup_storage(&mm, base);
    }

    /// Build a pending (un-finalized) receipt and place it in storage under
    /// `receipt_id`. `bls_certificate` is `None`, mimicking a Phase-A receipt.
    fn insert_pending_receipt(receipt_id: [u8; 32]) {
        let receipt = DeletionReceipt {
            protocol_version: ProtocolVersion::V3.into(),
            receipt_id,
            canister_id: Principal::anonymous(),
            record_id: vec![1, 2, 3],
            pre_state_hash: [0u8; 32],
            post_state_hash: [0u8; 32],
            tombstone_hash: [0u8; 32],
            deletion_event_hash: [0u8; 32],
            certified_commitment: [0u8; 32],
            module_hash: [0u8; 32],
            timestamp: 0,
            deletion_seq: 0,
            bls_certificate: None,
            trust_root_key_id: String::new(),
        };
        let mut buf = Vec::new();
        ciborium::into_writer(&receipt, &mut buf).unwrap();
        with_storage_mut(|s| {
            s.receipts.insert(Hash32(receipt_id), ReceiptBytes(buf));
        });
    }

    fn load_receipt(receipt_id: [u8; 32]) -> DeletionReceipt {
        let bytes = with_storage(|s| s.receipts.get(&Hash32(receipt_id))).unwrap();
        ciborium::from_reader(bytes.0.as_slice()).unwrap()
    }

    #[test]
    fn pending_identity_helper_uses_persisted_value_not_deletion_seq() {
        setup_test_storage(100);
        acquire_finalization_lock();

        let pending_id = [0xAB; 32];
        set_pending_receipt_id(pending_id);
        let before_seq = increment_deletion_seq();
        let after_seq = increment_deletion_seq();
        assert!(after_seq > before_seq);

        let helper_id = read_pending_receipt_id();
        assert_eq!(helper_id, Some(pending_id));
        assert_eq!(pending_receipt_id(), Some(pending_id));

        release_finalization_lock();
    }

    // --- A2: host-authorized finalize (no controller check) ---------------
    // These exercise `finalize_receipt_after_host_authorization`, which shares
    // its entire body — via the private `finalize_locked_receipt` helper — with
    // the controller-guarded `finalize_receipt`. They run on the host because
    // this path touches no `ic_cdk` runtime calls.

    #[test]
    fn host_auth_finalize_succeeds_and_releases_lock() {
        setup_test_storage(110);
        let rid = [0x11; 32];
        insert_pending_receipt(rid);
        acquire_finalization_lock();
        set_pending_receipt_id(rid);

        let res = finalize_receipt_after_host_authorization(&rid, vec![0xAA, 0xBB]);
        assert!(res.is_ok(), "expected Ok, got {res:?}");
        assert!(!is_finalization_locked(), "lock must be released after finalize");

        let decoded = load_receipt(rid);
        assert_eq!(decoded.bls_certificate, Some(vec![0xAA, 0xBB]));
        assert_eq!(decoded.trust_root_key_id, zombie_core::nns_keys::active_key_id());
    }

    #[test]
    fn host_auth_mismatch_returns_receipt_id_mismatch() {
        setup_test_storage(111);
        let rid = [0x22; 32];
        insert_pending_receipt(rid);
        acquire_finalization_lock();
        set_pending_receipt_id(rid);

        let res = finalize_receipt_after_host_authorization(&[0x33; 32], vec![0x01]);
        assert!(
            matches!(res, Err(FinalizationError::ReceiptIdMismatch { .. })),
            "got {res:?}"
        );
        assert!(is_finalization_locked(), "lock must stay held on mismatch");
    }

    #[test]
    fn host_auth_no_lock_returns_no_pending_receipt() {
        setup_test_storage(112);
        // No lock acquired, no pending id set.
        let res = finalize_receipt_after_host_authorization(&[0x44; 32], vec![0x01]);
        assert!(matches!(res, Err(FinalizationError::NoPendingReceipt)), "got {res:?}");
    }

    #[test]
    fn host_auth_double_finalize_returns_already_finalized() {
        setup_test_storage(113);
        let rid = [0x55; 32];
        insert_pending_receipt(rid);
        acquire_finalization_lock();
        set_pending_receipt_id(rid);

        // First finalize succeeds and releases the lock.
        finalize_receipt_after_host_authorization(&rid, vec![0x01]).expect("first finalize ok");

        // Re-acquire the lock pointing at the now-finalized receipt; the
        // second attempt must hit the already-finalized guard.
        acquire_finalization_lock();
        set_pending_receipt_id(rid);
        let res = finalize_receipt_after_host_authorization(&rid, vec![0x02]);
        assert!(matches!(res, Err(FinalizationError::AlreadyFinalized)), "got {res:?}");
    }
}
