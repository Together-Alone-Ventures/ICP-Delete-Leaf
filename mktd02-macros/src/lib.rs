//! # MKTd02 Procedural Macros
//!
//! ## `#[mktd_guard]`
//!
//! Injects tombstone + initialisation checks at the top of a function body.
//! Uses the `GuardError` trait: returns `Err(E::tombstone_violation())` or
//! `Err(E::not_initialised())`.
//!
//! Works with any `Result<T, E> where E: GuardError`.
//!
//! **Non-macro alternative:** `mktd02::assert_can_write()` for functions
//! returning `()` or preferring trap semantics.

extern crate proc_macro;

// TODO(Phase 3.1): #[mktd_guard] proc macro implementation
