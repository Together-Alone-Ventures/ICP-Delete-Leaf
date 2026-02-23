//! # Stable Memory Layout Manager
//!
//! 8 slots with configurable base MemoryId (default: 100).
//!
//! | Offset  | Content              | Type                                  |
//! |---------|----------------------|---------------------------------------|
//! | base+0  | Meta cell            | schema_version, memory_base, init_at, module_hash |
//! | base+1  | state_hash           | [u8; 32]                              |
//! | base+2  | nonce                | u64                                   |
//! | base+3  | certified_commitment | [u8; 32]                              |
//! | base+4  | deletion_event_hash  | [u8; 32]                              |
//! | base+5  | manifest_hash        | [u8; 32]                              |
//! | base+6  | receipt store        | StableBTreeMap<[u8;32], Vec<u8>>      |
//! | base+7  | tombstoned_at        | Option<u64>                           |
//!
//! Range check on init: base + 7 <= 255, else trap.
//! Collision detection: meta cell stores chosen base; re-init with
//! different base is a hard fail.

// TODO(Phase 2.2): Storage layout, meta cell, range check, collision detection
