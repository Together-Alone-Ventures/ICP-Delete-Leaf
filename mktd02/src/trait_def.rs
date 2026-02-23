//! # MKTdDataSource Trait & GuardError Trait
//!
//! The adapter trait that host canisters implement to integrate with MKTd02.
//!
//! - `MKTdDataSource`: mode(), pii_field_manifest(), manifest_hash(),
//!   get_state_bytes(), tombstone_state(), is_tombstoned()
//! - `CommitMode`: Leaf | Tree
//! - `GuardError`: tombstone_violation() -> Self, not_initialised() -> Self
//!   — implemented by the host canister's error type.

// TODO(Phase 2.1): MKTdDataSource trait, CommitMode enum, GuardError trait
