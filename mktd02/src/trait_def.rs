//! # MKTdDataSource Trait & GuardError Trait
//!
//! The adapter trait that host canisters implement to integrate with MKTd02.
//!
//! ## v0.2.0 Changes
//!
//! - Removed `manifest_hash()` — PII boundary is now anchored by
//!   module_hash → archived source code, not a dedicated manifest hash.

use zombie_core::FieldDescriptor;

/// Commit mode for CVDR generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitMode {
    /// Single data subject per canister (MKTd02).
    Leaf,
    /// Multiple data subjects per canister (MKTd03).
    Tree,
}

impl CommitMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommitMode::Leaf => "Leaf",
            CommitMode::Tree => "Tree",
        }
    }
}

/// The adapter trait that host canisters implement to integrate with MKTd02.
///
/// Each method maps to a specific aspect of the PII lifecycle:
/// - `mode()`: Declares Leaf or Tree commit mode
/// - `pii_field_manifest()`: Defines the PII boundary (documentation obligation)
/// - `get_state_bytes()`: Current PII state as deterministic CBOR bytes
/// - `tombstone_state()`: Overwrites all PII fields with TOMBSTONE_CONSTANT
/// - `is_tombstoned()`: Checks whether all PII fields are tombstoned
///
/// **v0.2.0 note:** `manifest_hash()` has been removed. The PII boundary
/// is anchored by `module_hash` → archived source code → adapter implementation.
/// Enterprises must publish a State Encoding Spec as a documentation obligation.
pub trait MKTdDataSource {
    fn mode(&self) -> CommitMode;
    fn pii_field_manifest(&self) -> Vec<FieldDescriptor>;

    /// Return deterministic CBOR bytes of current PII state.
    /// **Must** use `zombie_core::encode_pii_state()`.
    fn get_state_bytes(&self) -> Vec<u8>;

    /// Overwrite all PII fields with TOMBSTONE_CONSTANT.
    fn tombstone_state(&mut self);

    /// Post-condition check: all PII fields == TOMBSTONE_CONSTANT.
    fn is_tombstoned(&self) -> bool;
}

/// Trait for host canister error types to support the `#[mktd_guard]` macro.
///
/// The host canister's error enum implements this to provide typed
/// error variants for tombstone and initialisation violations.
pub trait GuardError {
    fn tombstone_violation() -> Self;
    fn not_initialised() -> Self;
}
