//! # Reference Adapter: StableBTreeMap Pattern
//!
//! Reference `MKTdDataSource` implementation for a canister using
//! `StableBTreeMap` with user profile structs.
//!
//! This adapter pattern is for **MKTd03 (Tree mode)** where a single
//! canister holds multiple data subjects' records. It is included here
//! as a reference for future MKTd03 integration work.
//!
//! For MKTd02 (Leaf mode, single data subject per canister), use the
//! `adapter_stable_cell.rs` pattern instead.

// NOTE: This is a reference stub for MKTd03. The Tree-mode adapter
// requires per-record operations (get_record, tombstone_record,
// is_record_tombstoned) rather than whole-canister operations.
// Full implementation deferred to MKTd03 build.
