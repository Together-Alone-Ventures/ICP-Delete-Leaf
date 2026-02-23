//! # Reference Adapter: StableCell Pattern
//!
//! Reference `MKTdDataSource` implementation for a canister using
//! `StableCell` with a profile struct (matches DaffyDefs pattern).
//!
//! Uses `encode_pii_state()` for `get_state_bytes()`.
//!
//! ## Canonicalisation Checklist
//! - [ ] All PII fields listed in `pii_field_manifest()` in consistent order
//! - [ ] `get_state_bytes()` uses `encode_pii_state()` (deterministic CBOR)
//! - [ ] `tombstone_state()` writes TOMBSTONE_CONSTANT to every PII field
//! - [ ] `is_tombstoned()` checks all PII fields == TOMBSTONE_CONSTANT
//! - [ ] Field order in manifest matches field order in serialisation

// TODO(Phase 4.1): Reference adapter implementation
