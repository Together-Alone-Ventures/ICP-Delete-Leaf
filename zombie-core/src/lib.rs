//! # zombie-core
//!
//! Shared types, hashing primitives, and receipt structures for the
//! Zombie Delete CVDR (Cryptographically Verifiable Deletion Receipt) system.
//!
//! This crate is pure Rust with zero ICP dependencies. It compiles and
//! tests on native targets.

pub mod hashing;
pub mod manifest;
pub mod receipt;
pub mod serialisation;
pub mod tombstone;
