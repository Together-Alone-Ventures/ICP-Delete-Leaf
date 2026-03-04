//! # Receipt Finalization (Phase B + Phase C)
//!
//! Implements the two post-deletion steps that embed the BLS certificate
//! and NNS root key into a pending receipt, making it self-contained
//! for offline V2 verification.
//!
//! ## Three-Phase Deletion Flow
//!
//! | Phase | Call Type | What Happens                                        |
//! |-------|-----------|-----------------------------------------------------|
//! | A     | update    | `execute_deletion()` — tombstone, emit receipt,     |
//! |       |           | publish certified data, acquire finalization lock    |
//! | B     | query     | `get_pending_certificate()` — read BLS certificate  |
//! |       |           | from IC runtime (only available in query context)    |
//! | C     | update    | `finalize_receipt()` — embed certificate + root key |
//! |       |           | in receipt, release finalization lock                |
//!
//! ## Orchestration
//!
//! The library provides the three functions. How they are called is up
//! to the integrator:
//!
//! - **Factory pattern** (recommended for multi-canister apps):
//!   The factory makes three inter-canister calls in sequence.
//! - **Script pattern** (simplest for testing / single-canister):
//!   A shell or JS script runs three `dfx` calls in sequence.
//! - **Timer pattern** (self-contained canister):
//!   The canister schedules a self-call timer after Phase A.
//!
//! See the Integration Guide for example orchestration code.

use crate::certified::read_certified_commitment;
use crate::nonce::current_nonce;
use crate::storage::{with_storage, with_storage_mut, Hash32, ReceiptBytes};
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
/// The orchestrator should pass `certificate` and the NNS root key
/// to `finalize_receipt()` in Phase C.
pub fn get_pending_certificate() -> Option<PendingCertificate> {
    // Check finalization lock
    if !crate::storage::is_finalization_locked() {
        return None;
    }

    // Compute the pending receipt_id from canister_id + current nonce
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

/// Phase C: Embed the BLS certificate and NNS root key into the pending receipt.
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
/// - `trust_root_key`: The NNS root public key (96 bytes for mainnet).
///   The orchestrator provides this from configuration — it is NOT
///   extracted from the certificate by the library.
///
/// On success, the receipt's `bls_certificate` and `trust_root_key`
/// fields are populated and the finalization lock is released. The
/// receipt is now self-contained for offline V2 verification.
pub fn finalize_receipt(
    receipt_id: &[u8; 32],
    certificate: Vec<u8>,
    trust_root_key: Vec<u8>,
) -> Result<(), FinalizationError> {
    // Guard 1: finalization lock must be held
    if !crate::storage::is_finalization_locked() {
        return Err(FinalizationError::NoPendingReceipt);
    }

    // Guard 2: caller must be a controller
    let caller = ic_cdk::caller();
    if !ic_cdk::api::is_controller(&caller) {
        return Err(FinalizationError::NotController);
    }

    // Compute expected receipt_id from canister_id + current nonce
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

    // Embed certificate and root key
    receipt.bls_certificate = Some(certificate);
    receipt.trust_root_key = trust_root_key;

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
