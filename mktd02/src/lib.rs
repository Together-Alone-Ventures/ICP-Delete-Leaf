//! # MKTd02 -- Leaf-Mode CVDR Engine
//!
//! A composable Rust library that any ICP canister can import to produce
//! CVDRs (Cryptographically Verifiable Deletion Receipts) for GDPR
//! right-to-erasure compliance.
//!
//! ## Quick Start
//!
//! 1. Implement `MKTdDataSource` for your canister's data layer
//! 2. Call `mktd02::init()` in `#[init]` or first `post_upgrade`
//! 3. Call `mktd02::on_post_upgrade()` in every `#[post_upgrade]`
//! 4. Guard PII-mutating functions with `#[mktd_guard]` or `assert_can_write()`
//! 5. Call `mktd02::refresh_state_hash()` after each PII mutation
//! 6. Call `mktd02::execute_deletion()` to tombstone and generate a CVDR

pub mod certified;
pub mod engine;
pub mod export;
pub mod guard;
pub mod nonce;
pub mod state;
pub mod storage;
pub mod trait_def;

// --- Re-exports ---
pub use engine::DeletionError;
pub use trait_def::{CommitMode, GuardError, MKTdDataSource};
pub use zombie_core::{DeletionReceipt, FieldDescriptor, ReceiptSummary};

use candid::Principal;
use ic_stable_structures::memory_manager::MemoryManager;
use ic_stable_structures::DefaultMemoryImpl;
use storage::Hash32;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for MKTd02 initialisation.
pub struct MktdConfig {
    /// Base MemoryId for MKTd02's 8 stable memory slots (default: 100).
    /// Range: base + 7 must be <= 255.
    pub base_memory_id: u8,
    /// Subnet ID for receipt construction.
    pub subnet_id: Principal,
}

impl Default for MktdConfig {
    fn default() -> Self {
        Self {
            base_memory_id: 100,
            subnet_id: Principal::anonymous(),
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// First-time initialisation. Call from `#[init]`.
///
/// Sets up stable memory, computes initial state hash, publishes
/// certified commitment. Module hash is provided by the deployer
/// (see Module Hash: Deployment Patterns in the Integration Guide).
pub fn init<A: MKTdDataSource>(
    adapter: &A,
    memory_manager: &MemoryManager<DefaultMemoryImpl>,
    config: MktdConfig,
    module_hash: [u8; 32],
) {
    storage::setup_storage(memory_manager, config.base_memory_id);
    if guard::is_initialised() {
        return;
    }
    engine::first_init(adapter, &config, module_hash);
}

/// Post-upgrade handler. Call from `#[post_upgrade]`.
///
/// Reconnects to stable memory, runs manifest-change cascade,
/// updates module_hash unconditionally.
pub fn on_post_upgrade<A: MKTdDataSource>(
    adapter: &A,
    memory_manager: &MemoryManager<DefaultMemoryImpl>,
    config: MktdConfig,
    module_hash: [u8; 32],
) {
    storage::setup_storage(memory_manager, config.base_memory_id);

    // If not yet initialised (upgrade that adds MKTd02), run first_init.
    if !guard::is_initialised() {
        engine::first_init(adapter, &config, module_hash);
    }

    engine::upgrade_cascade(adapter, module_hash);
}

/// Execute deletion: tombstone PII and generate a CVDR.
///
/// Returns the 32-byte receipt_id on success.
pub fn execute_deletion<A: MKTdDataSource>(
    adapter: &mut A,
    config: &MktdConfig,
) -> Result<[u8; 32], DeletionError> {
    engine::execute_deletion(adapter, config)
}

/// Check whether the canister is tombstoned.
pub fn is_tombstoned() -> bool {
    guard::is_tombstoned()
}

/// Check whether MKTd02 has been initialised.
pub fn is_initialised() -> bool {
    guard::is_initialised()
}

/// Get the current state hash.
pub fn get_state_hash() -> [u8; 32] {
    state::read_state_hash()
}

/// Get state hash with optional ICP certificate (for certified queries).
pub fn get_certified_state_hash() -> ([u8; 32], Option<Vec<u8>>) {
    certified::get_certified_state_hash()
}

/// Retrieve a stored receipt by ID.
pub fn get_receipt(receipt_id: &[u8; 32]) -> Option<DeletionReceipt> {
    storage::with_storage(|s| {
        s.receipts.get(&Hash32(*receipt_id)).and_then(|bytes| {
            ciborium::from_reader(bytes.0.as_slice()).ok()
        })
    })
}

/// Retrieve a lightweight receipt summary by ID.
pub fn get_receipt_summary(receipt_id: &[u8; 32]) -> Option<ReceiptSummary> {
    get_receipt(receipt_id).map(|r| ReceiptSummary::from(&r))
}

/// Get tombstone status (timestamp if tombstoned).
pub fn get_tombstone_status() -> Option<u64> {
    storage::with_storage(|s| s.tombstoned_at.get().0)
}

/// Recompute state hash after a PII mutation.
///
/// Call this after every write to PII fields (e.g., after upsert_profile).
pub fn refresh_state_hash<A: MKTdDataSource>(adapter: &A) {
    state::refresh_state_hash_internal(&adapter.get_state_bytes());

    // Re-publish certified commitment with updated state_hash
    let new_state_hash = state::read_state_hash();
    let existing_event_hash =
        storage::with_storage(|s| s.deletion_event_hash.get().0);
    certified::publish_certified_commitment(&new_state_hash, &existing_event_hash);
}

/// Get the number of stored receipts.
pub fn receipt_count() -> u64 {
    storage::with_storage(|s| s.receipts.len())
}

/// Trap if not initialised or tombstoned. For non-Result functions.
pub fn assert_can_write() {
    guard::assert_can_write();
}
