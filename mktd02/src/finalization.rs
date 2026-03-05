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
//! ## Nonce Invariant
//!
//! Phase B and Phase C both derive the pending receipt_id from
//! (canister_id, current_nonce()). This is correct only while the
//! finalization lock is held — the lock is acquired in Phase A and
//! released at the end of Phase C. No code path must advance the nonce
//! while the lock is held.
//!
//! HARDENING TODO: store the pending receipt_id in a dedicated stable-memory
//! cell during Phase A, and read it here rather than recomputing. This removes
//! any risk if an integrator accidentally increments the nonce between phases.
//! Tracked as a follow-up to Change A.

use crate::certified::read_certified_commitment;
use crate::nonce::current_nonce;
use crate::storage::{with_storage, with_storage_mut, Hash32, ReceiptBytes};
use zombie_core::nns_keys;
use zombie_core::receipt::{compute_receipt_id, DeletionReceipt};

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

/// Phase B: Retrieve the BLS certificate for the pending receipt.
///
/// **Must be called in query context** — `ic0.data_certificate()` is
/// only available in query calls. Returns `None` if no receipt is
/// pending or if the IC runtime does not provide a certificate.
///
/// The orchestrator passes `certificate` to `finalize_receipt()` in Phase C.
/// The NNS root key ID is determined automatically by the library.
///
/// ## Nonce safety
///
/// receipt_id is derived from (canister_id, current_nonce()). This is safe
/// because the finalization lock — checked first — guarantees no other
/// deletion (which would advance the nonce) can run concurrently.
pub fn get_pending_certificate() -> Option<PendingCertificate> {
    // Check finalization lock — SAFETY: nonce must not change while lock is held
    if !crate::storage::is_finalization_locked() {
        return None;
    }

    // Derive pending receipt_id from canister_id + current nonce.
    // Correct only while finalization lock is held (see module-level note).
    let canister_id = ic_cdk::id();
    let nonce = current_nonce();
    let receipt_id = compute_receipt_id(&canister_id, nonce);

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
/// ## Nonce safety
///
/// expected_id is derived from (canister_id, current_nonce()). Correct only
/// while finalization lock is held — no path must advance the nonce between
/// Phase A and Phase C. See module-level HARDENING TODO.
pub fn finalize_receipt(
    receipt_id: &[u8; 32],
    certificate: Vec<u8>,
) -> Result<(), FinalizationError> {
    // Guard 1: finalization lock must be held — SAFETY: nonce stable while locked
    if !crate::storage::is_finalization_locked() {
        return Err(FinalizationError::NoPendingReceipt);
    }

    // Guard 2: caller must be a controller
    let caller = ic_cdk::caller();
    if !ic_cdk::api::is_controller(&caller) {
        return Err(FinalizationError::NotController);
    }

    // Derive expected receipt_id from canister_id + current nonce.
    // Safe: finalization lock ensures nonce has not advanced since Phase A.
    let canister_id = ic_cdk::id();
    let nonce = current_nonce();
    let expected_id = compute_receipt_id(&canister_id, nonce);

    // Guard 3: receipt_id must match
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

    // Guard 4: must not already be finalized
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
