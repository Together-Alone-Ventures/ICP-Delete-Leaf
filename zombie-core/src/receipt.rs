//! # CVDR (Cryptographically Verifiable Deletion Receipt)
//!
//! Receipt struct definition and `receipt_id` computation.
//! Domain tag: `MKTD02_RECEIPT_V1`
//!
//! The receipt is an **unsigned artifact**. Verification relies on
//! the certified commitment obtained via ICP's certified query
//! mechanism, not a signature on the receipt itself.

use crate::hashing::{hash_with_tag, TAG_RECEIPT};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct DeletionReceipt {
    pub receipt_id: [u8; 32],
    pub canister_id: Principal,
    pub subnet_id: Principal,
    pub commit_mode: String,
    pub pre_state_hash: [u8; 32],
    pub post_state_hash: [u8; 32],
    pub tombstone_hash: [u8; 32],
    pub deletion_event_hash: [u8; 32],
    pub certified_commitment: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub module_hash: [u8; 32],
    pub timestamp: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub struct ReceiptSummary {
    pub receipt_id: [u8; 32],
    pub canister_id: Principal,
    pub commit_mode: String,
    pub timestamp: u64,
    pub nonce: u64,
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

/// Compute a receipt ID.
///
/// `receipt_id = SHA-256(MKTD02_RECEIPT_V1 || canister_id_bytes || nonce_be_bytes)`
pub fn compute_receipt_id(canister_id: &Principal, nonce: u64) -> [u8; 32] {
    hash_with_tag(TAG_RECEIPT, &[canister_id.as_slice(), &nonce.to_be_bytes()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_id_deterministic() {
        let c = Principal::from_text("aaaaa-aa").unwrap();
        assert_eq!(compute_receipt_id(&c, 1), compute_receipt_id(&c, 1));
    }

    #[test]
    fn receipt_id_different_nonces_differ() {
        let c = Principal::from_text("aaaaa-aa").unwrap();
        assert_ne!(compute_receipt_id(&c, 1), compute_receipt_id(&c, 2));
    }

    #[test]
    fn receipt_id_different_canisters_differ() {
        let c1 = Principal::from_text("aaaaa-aa").unwrap();
        let c2 = Principal::from_text("2vxsx-fae").unwrap();
        assert_ne!(compute_receipt_id(&c1, 1), compute_receipt_id(&c2, 1));
    }

    #[test]
    fn receipt_summary_from_receipt() {
        let c = Principal::from_text("aaaaa-aa").unwrap();
        let r = DeletionReceipt {
            receipt_id: [1u8; 32], canister_id: c,
            subnet_id: Principal::from_text("2vxsx-fae").unwrap(),
            commit_mode: "Leaf".into(),
            pre_state_hash: [2u8; 32], post_state_hash: [3u8; 32],
            tombstone_hash: [4u8; 32], deletion_event_hash: [5u8; 32],
            certified_commitment: [6u8; 32], manifest_hash: [7u8; 32],
            module_hash: [8u8; 32], timestamp: 1_000_000, nonce: 1,
        };
        let s = ReceiptSummary::from(&r);
        assert!(s.state_changed);
        
    }
    #[test]
    fn receipt_summary_state_unchanged_when_equal() {
        let c = Principal::from_text("aaaaa-aa").unwrap();
        let r = DeletionReceipt {
            receipt_id: [1u8; 32], canister_id: c,
            subnet_id: Principal::from_text("2vxsx-fae").unwrap(),
            commit_mode: "Leaf".into(),
            pre_state_hash: [2u8; 32], post_state_hash: [2u8; 32],
            tombstone_hash: [4u8; 32], deletion_event_hash: [5u8; 32],
            certified_commitment: [6u8; 32], manifest_hash: [7u8; 32],
            module_hash: [8u8; 32], timestamp: 1_000_000, nonce: 1,
        };
        let s = ReceiptSummary::from(&r);
        assert!(!s.state_changed);
    }
    #[test]
    fn golden_receipt_id() {
        // aaaaa-aa (management canister), nonce = 1
        let c = Principal::from_text("aaaaa-aa").unwrap();
        let id = compute_receipt_id(&c, 1);
        assert_eq!(
            hex::encode(id),
            "10e9cc19646743d46a2dda9e535f7a8389635be0559a4980dca46c444661be02",
            "receipt_id derivation changed — this breaks all existing receipts"
        );
    }
}
