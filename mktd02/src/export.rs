//! # Receipt Export
//!
//! - `to_cbor_bytes()` — always available; canonical CBOR encoding
//! - `to_json()` — behind `#[cfg(feature = "json")]`; for SIEM/GRC ingestion
//! - `webhook_push()` — behind `#[cfg(feature = "json")]`; HTTP outcall template

// TODO(Phase 2.7): to_cbor_bytes, to_json, webhook_push
