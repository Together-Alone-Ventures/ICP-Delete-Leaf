//! # Hashing Primitives & Domain Separation
//!
//! SHA-256 wrapper, domain separation tags, and byte concatenation helpers.
//!
//! ## Naming Convention Table
//!
//! To prevent confusion between similarly-named values that serve different purposes:
//!
//! | Name                     | Kind           | Purpose                                      | Used in          |
//! |--------------------------|----------------|----------------------------------------------|------------------|
//! | `MKTD_TOMBSTONE_V1`      | Constant seed  | Seed for TOMBSTONE_CONSTANT (bytes written)   | tombstone.rs     |
//! | `MKTD02_TOMBSTONE_HASH_V1` | Domain tag  | Tag for tombstone_hash in receipt             | engine.rs        |
//! | `MKTD02_EVENT_V1`        | Domain tag     | Tag for deletion_event_hash                   | engine.rs        |
//! | `MKTD02_CERTIFIED_V1`    | Domain tag     | Tag for certified_commitment                  | certified.rs     |
//! | `MKTD02_RECEIPT_V1`      | Domain tag     | Tag for receipt_id derivation                 | receipt.rs       |
//! | `MKTD02_SALT_V1`         | Domain tag     | Tag for per-canister salt derivation          | state.rs         |
//! | `MKTD02_MANIFEST_V1`     | Domain tag     | Tag for manifest_hash computation             | manifest.rs      |
//!
//! **Key distinction:** The tombstone constant is a *value written to storage*;
//! domain tags are *prefixes for hash computations*.

// TODO(Phase 1.1): SHA-256 wrapper, domain separation tags, byte concatenation helpers
