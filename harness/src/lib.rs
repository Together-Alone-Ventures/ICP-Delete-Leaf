//! Minimal **test-only** canister wrapping the MKTd02 engine entry points, so
//! the three runtime-coupled P0 cases can be driven under PocketIC:
//!
//! 1. caller-derived `record_id` (`execute_deletion`)
//! 2. host-supplied `record_id` (`execute_deletion_with_record_id`)
//! 3. controller-guard ordering on the default `finalize_receipt` path
//!
//! This is NOT a production canister. It deliberately does **not** expose
//! `finalize_receipt_after_host_authorization`, and it emits no `.did` — so it
//! introduces no Candid surface for the host-auth path. A tiny single-field
//! PII adapter satisfies `MKTdDataSource`; it is throwaway test scaffolding,
//! not the production integration adapter.

use std::cell::RefCell;

use candid::{CandidType, Principal};
use ic_stable_structures::memory_manager::MemoryManager;
use ic_stable_structures::DefaultMemoryImpl;
use mktd02::trait_def::{CommitMode, MKTdDataSource};
use mktd02::{FinalizationError, MktdConfig};
use serde::{Deserialize, Serialize};
use zombie_core::serialisation::encode_pii_state;
use zombie_core::tombstone::tombstone_constant;
use zombie_core::FieldDescriptor;

#[derive(Serialize, Deserialize)]
struct PiiState {
    email: String,
}

/// Trivial single-PII-field adapter for the harness canister.
struct TestAdapter {
    email: String,
    tombstoned: bool,
}

impl TestAdapter {
    fn new() -> Self {
        Self {
            email: "alice@example.com".to_string(),
            tombstoned: false,
        }
    }
}

impl MKTdDataSource for TestAdapter {
    fn mode(&self) -> CommitMode {
        CommitMode::Leaf
    }

    fn pii_field_manifest(&self) -> Vec<FieldDescriptor> {
        vec![FieldDescriptor {
            field_name: "email".into(),
            field_type: "String".into(),
            field_order: 0,
        }]
    }

    fn get_state_bytes(&self) -> Vec<u8> {
        encode_pii_state(&PiiState {
            email: self.email.clone(),
        })
        .expect("encode_pii_state failed")
    }

    fn tombstone_state(&mut self) {
        self.email = hex::encode(tombstone_constant());
        self.tombstoned = true;
    }

    fn is_tombstoned(&self) -> bool {
        self.tombstoned
    }
}

thread_local! {
    static MM: MemoryManager<DefaultMemoryImpl> = MemoryManager::init(DefaultMemoryImpl::default());
    static ADAPTER: RefCell<TestAdapter> = RefCell::new(TestAdapter::new());
}

#[ic_cdk::init]
fn init() {
    let module_hash = [0x11u8; 32];
    ADAPTER.with(|a| {
        MM.with(|mm| {
            mktd02::init(&*a.borrow(), mm, MktdConfig::default(), module_hash);
        });
    });
}

/// Phase A via the caller-derived `record_id` wrapper.
#[ic_cdk::update]
fn h_execute_deletion() -> Result<Vec<u8>, String> {
    ADAPTER.with(|a| {
        let mut a = a.borrow_mut();
        mktd02::execute_deletion(&mut *a, &MktdConfig::default())
            .map(|id| id.to_vec())
            .map_err(|e| e.to_string())
    })
}

/// Phase A via the host-supplied opaque `record_id`.
#[ic_cdk::update]
fn h_execute_deletion_with_record_id(record_id: Vec<u8>) -> Result<Vec<u8>, String> {
    ADAPTER.with(|a| {
        let mut a = a.borrow_mut();
        mktd02::execute_deletion_with_record_id(&mut *a, &MktdConfig::default(), record_id)
            .map(|id| id.to_vec())
            .map_err(|e| e.to_string())
    })
}

/// Default (controller-guarded) Phase C path. Returns the error *variant name*
/// so the driver can assert guard ordering precisely.
#[ic_cdk::update]
fn h_finalize_receipt(receipt_id: Vec<u8>, certificate: Vec<u8>) -> String {
    let mut id = [0u8; 32];
    if receipt_id.len() == 32 {
        id.copy_from_slice(&receipt_id);
    }
    match mktd02::finalize_receipt(&id, certificate) {
        Ok(()) => "Ok".to_string(),
        Err(FinalizationError::NotController) => "NotController".to_string(),
        Err(FinalizationError::NoPendingReceipt) => "NoPendingReceipt".to_string(),
        Err(FinalizationError::ReceiptIdMismatch { .. }) => "ReceiptIdMismatch".to_string(),
        Err(FinalizationError::AlreadyFinalized) => "AlreadyFinalized".to_string(),
        Err(FinalizationError::ReceiptNotFound) => "ReceiptNotFound".to_string(),
        Err(FinalizationError::EncodingFailed(_)) => "EncodingFailed".to_string(),
    }
}

/// Receipt fields the driver needs to verify the record_id / receipt_id /
/// certified-proof trace. Primitive types only (no dependence on the receipt's
/// own Candid-ness).
#[derive(CandidType, Deserialize)]
pub struct ReceiptFieldsDto {
    pub record_id: Vec<u8>,
    pub receipt_id: Vec<u8>,
    pub deletion_seq: u64,
    pub canister_id: Principal,
    pub pre_state_hash: Vec<u8>,
    pub post_state_hash: Vec<u8>,
    pub deletion_event_hash: Vec<u8>,
    pub certified_commitment: Vec<u8>,
}

#[ic_cdk::query]
fn h_get_receipt_fields(receipt_id: Vec<u8>) -> Option<ReceiptFieldsDto> {
    if receipt_id.len() != 32 {
        return None;
    }
    let mut id = [0u8; 32];
    id.copy_from_slice(&receipt_id);
    mktd02::get_receipt(&id).map(|r| ReceiptFieldsDto {
        record_id: r.record_id,
        receipt_id: r.receipt_id.to_vec(),
        deletion_seq: r.deletion_seq,
        canister_id: r.canister_id,
        pre_state_hash: r.pre_state_hash.to_vec(),
        post_state_hash: r.post_state_hash.to_vec(),
        deletion_event_hash: r.deletion_event_hash.to_vec(),
        certified_commitment: r.certified_commitment.to_vec(),
    })
}
