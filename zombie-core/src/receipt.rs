//! # CVDR (Cryptographically Verifiable Deletion Receipt)
//!
//! Receipt struct definition and `receipt_id` computation.
//! Domain tag: `MKTD02_RECEIPT_V1`
//!
//! The receipt is an **unsigned artifact**. Verification relies on
//! the certified commitment obtained via ICP's certified query
//! mechanism, not a signature on the receipt itself.

use crate::hashing::{sha256_concat, TAG_RECEIPT};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// A Cryptographically Verifiable Deletion Receipt (CVDR).
///
/// Contains all fields needed for independent verification of a
/// deletion event. All hash fields are `[u8; 32]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct DeletionReceipt {
    /// Unique receipt identifier.
    /// `SHA-256(MKTD02_RECEIPT_V1 || canister_id || nonce)`
    pub receipt_id: [u8; 32],

    /// Principal of the canister that produced this receipt.
    pub canister_id: Principal,

    /// Subnet hosting the canister at time of deletion.
    pub subnet_id: Principal,

    /// Commit mode used: "Leaf" for MKTd02.
    pub commit_mode: String,

    /// State hash of PII data *before* tombstoning.
    pub pre_state_hash: [u8; 32],

    /// State hash of PII data *after* tombstoning.
    pub post_state_hash: [u8; 32],

    /// `SHA-256(MKTD02_TOMBSTONE_HASH_V1 || canister_id || TOMBSTONE_CONSTANT || timestamp || nonce)`
    pub tombstone_hash: [u8; 32],

    /// `SHA-256(MKTD02_EVENT_V1 || pre_state_hash || post_state_hash || timestamp || module_hash || manifest_hash || nonce)`
    pub deletion_event_hash: [u8; 32],

    /// `SHA-256(MKTD02_CERTIFIED_V1 || post_state_hash || deletion_event_hash)`
    pub certified_commitment: [u8; 32],

    /// Hash of the PII field manifest at time of deletion.
    pub manifest_hash: [u8; 32],

    /// Canister WASM module hash at time of deletion.
    pub module_hash: [u8; 32],

    /// ICP system time at deletion (nanoseconds since epoch).
    pub timestamp: u64,

    /// Monotonic nonce, unique per receipt, never reused.
    pub nonce: u64,
}

/// A lightweight receipt summary suitable for listing/display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct ReceiptSummary {
    pub receipt_id: [u8; 32],
    pub canister_id: Principal,
    pub commit_mode: String,
    pub timestamp: u64,
    pub nonce: u64,
    /// Whether pre_state_hash != post_state_hash (basic sanity).
    pub state_changed: bool,
}

impl From<&DeletionReceipt> for ReceiptSummary {
    fn from(r: &DeletionReceipt) -> Self {
        Self {
            receipt_id: r.receipt_id,
            canister_id: r.canister_id,
            commit_mode: r.commit_mode.clone(),
            timestamp: r.timestamp,
            nonce: r.nonce,
            state_changed: r.pre_state_hash != r.post_state_hash,
        }
    }
}

/// Compute a receipt ID from its components.
///
/// `receipt_id = SHA-256(MKTD02_RECEIPT_V1 || canister_id_bytes || nonce_be_bytes)`
pub fn compute_receipt_id(canister_id: &Principal, nonce: u64) -> [u8; 32] {
    sha256_concat(&[
        TAG_RECEIPT,
        canister_id.as_slice(),
        &nonce.to_be_bytes(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_id_deterministic() {
        let canister = Principal::from_text("aaaaa-aa").unwrap();
        let a = compute_receipt_id(&canister, 1);
        let b = compute_receipt_id(&canister, 1);
        assert_eq!(a, b);
    }

    #[test]
    fn receipt_id_different_nonces_differ() {
        let canister = Principal::from_text("aaaaa-aa").unwrap();
        let a = compute_receipt_id(&canister, 1);
        let b = compute_receipt_id(&canister, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn receipt_id_different_canisters_differ() {
        let c1 = Principal::from_text("aaaaa-aa").unwrap();
        let c2 = Principal::from_text("2vxsx-fae").unwrap();
        let a = compute_receipt_id(&c1, 1);
        let b = compute_receipt_id(&c2, 1);
        assert_ne!(a, b);
    }

    #[test]
    fn receipt_summary_from_receipt() {
        let canister = Principal::from_text("aaaaa-aa").unwrap();
        let receipt = DeletionReceipt {
            receipt_id: [1u8; 32],
            canister_id: canister,
            subnet_id: Principal::from_text("2vxsx-fae").unwrap(),
            commit_mode: "Leaf".into(),
            pre_state_hash: [2u8; 32],
            post_state_hash: [3u8; 32],
            tombstone_hash: [4u8; 32],
            deletion_event_hash: [5u8; 32],
            certified_commitment: [6u8; 32],
            manifest_hash: [7u8; 32],
            module_hash: [8u8; 32],
            timestamp: 1_000_000,
            nonce: 1,
        };
        let summary = ReceiptSummary::from(&receipt);
        assert_eq!(summary.receipt_id, receipt.receipt_id);
        assert_eq!(summary.canister_id, receipt.canister_id);
        assert!(summary.state_changed);
    }

    #[test]
    fn receipt_summary_state_unchanged_when_equal() {
        let canister = Principal::from_text("aaaaa-aa").unwrap();
        let receipt = DeletionReceipt {
            receipt_id: [1u8; 32],
            canister_id: canister,
            subnet_id: Principal::from_text("2vxsx-fae").unwrap(),
            commit_mode: "Leaf".into(),
            pre_state_hash: [2u8; 32],
            post_state_hash: [2u8; 32],
            tombstone_hash: [4u8; 32],
            deletion_event_hash: [5u8; 32],
            certified_commitment: [6u8; 32],
            manifest_hash: [7u8; 32],
            module_hash: [8u8; 32],
            timestamp: 1_000_000,
            nonce: 1,
        };
        let summary = ReceiptSummary::from(&receipt);
        assert!(!summary.state_changed);
    }
}
