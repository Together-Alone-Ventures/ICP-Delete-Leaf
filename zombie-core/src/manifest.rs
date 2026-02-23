//! # PII Field Manifest
//!
//! `FieldDescriptor` and `manifest_hash` computation.
//! Domain tag: `MKTD02_MANIFEST_V1`
//!
//! The manifest defines the PII boundary — which fields are covered
//! by MKTd02. Changes to the manifest during upgrade trigger a full
//! recomputation cascade (state_hash → certified_commitment → publish).

// TODO(Phase 1.4): FieldDescriptor struct, manifest_hash()
