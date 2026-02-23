//! # MKTd02 — Leaf-Mode CVDR Engine
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

// --- Public API (Phase 2.9) ---
// TODO: init(), on_post_upgrade(), execute_deletion()
// TODO: is_tombstoned(), is_initialised(), get_state_hash()
// TODO: get_certified_state_hash(), get_receipt(), get_receipt_summary()
// TODO: get_tombstone_status(), refresh_state_hash(), receipt_count()
// TODO: assert_can_write()

// --- Re-exports ---
pub use trait_def::{CommitMode, GuardError, MKTdDataSource};
// TODO: Re-export DeletionReceipt, ReceiptSummary, FieldDescriptor, MktdConfig
//       from zombie_core / local types once defined.
