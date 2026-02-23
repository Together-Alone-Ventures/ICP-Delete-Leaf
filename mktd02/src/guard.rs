//! # Tombstone & Initialisation Guards
//!
//! - `is_tombstoned()` — reads tombstoned_at from stable memory
//! - `is_initialised()` — checks meta cell for initialised state
//! - `assert_can_write()` — traps with descriptive error if tombstoned
//!   or not initialised. For use in non-Result functions preferring
//!   trap-on-error semantics.

// TODO(Phase 2.8): is_tombstoned, is_initialised, assert_can_write
