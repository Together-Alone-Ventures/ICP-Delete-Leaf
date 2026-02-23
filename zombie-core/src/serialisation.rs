//! # Deterministic CBOR Serialisation
//!
//! `encode_pii_state()` — safe CBOR encoding with built-in validation.
//!
//! This module provides the canonical serialisation used by adapters
//! to produce `get_state_bytes()`. Deterministic CBOR is critical:
//! the same logical state must always produce the same bytes, because
//! those bytes are hashed to produce the state_hash.

// TODO(Phase 1.5): encode_pii_state(), validation helpers
