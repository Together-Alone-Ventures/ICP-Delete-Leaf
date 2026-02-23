//! # Tombstone Constant
//!
//! `TOMBSTONE_CONSTANT: [u8; 32] = SHA-256("MKTD_TOMBSTONE_V1")`
//!
//! This is the value written to each PII field during tombstoning.
//! It is a well-known, deterministic value that any verifier can
//! independently recompute from the published seed string.
//!
//! **This is NOT a domain separation tag.** It is the actual bytes
//! that replace PII data in storage.

// TODO(Phase 1.2): TOMBSTONE_CONSTANT, computed at build time or lazy_static
