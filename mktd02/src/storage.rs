//! # Stable Memory Layout Manager
//!
//! 8 slots with configurable base MemoryId (default: 100).
//!
//! | Offset  | Content              | Type                                          |
//! |---------|----------------------|-----------------------------------------------|
//! | base+0  | Meta cell            | schema_version, memory_base, init_at, mod_hash|
//! | base+1  | state_hash           | [u8; 32]                                      |
//! | base+2  | nonce                | u64                                           |
//! | base+3  | certified_commitment | [u8; 32]                                      |
//! | base+4  | deletion_event_hash  | [u8; 32]                                      |
//! | base+5  | manifest_hash        | [u8; 32]                                      |
//! | base+6  | receipt store        | StableBTreeMap<[u8;32], Vec<u8>>              |
//! | base+7  | tombstoned_at        | Option<u64>                                   |
//!
//! Range check on init: base + 7 <= 255, else trap.
//!
//! ## Endianness Convention
//!
//! - **Stable memory encoding** (Storable impls): little-endian (LE).
//!   This matches ICP's native memory representation.
//! - **Hash preimages** (in hashing.rs, engine.rs, etc.): big-endian (BE).
//!   This follows cryptographic convention for unambiguous byte ordering.
//!
//! These are separate domains and must not be confused.

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use std::borrow::Cow;
use std::cell::RefCell;

pub(crate) type Memory = VirtualMemory<DefaultMemoryImpl>;

// ---------------------------------------------------------------------------
// Storable wrapper types
// ---------------------------------------------------------------------------

/// 32-byte hash wrapper with Storable + Ord for use as StableBTreeMap key.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Hash32(pub [u8; 32]);

impl Storable for Hash32 {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0)
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Self(arr)
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 32,
        is_fixed_size: true,
    };
}

/// Meta cell: schema_version(4) + memory_base(4) + has_init(1) + init_at(8) + module_hash(32) = 49 bytes
///
/// All integer fields use little-endian encoding (stable memory convention).
#[derive(Clone, Debug)]
pub(crate) struct MetaCell {
    pub schema_version: u32,
    pub memory_base: u32,
    pub initialised_at: Option<u64>,
    pub module_hash: [u8; 32],
}

impl Default for MetaCell {
    fn default() -> Self {
        Self {
            schema_version: 0,
            memory_base: 0,
            initialised_at: None,
            module_hash: [0u8; 32],
        }
    }
}

impl Storable for MetaCell {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![0u8; 49];
        buf[0..4].copy_from_slice(&self.schema_version.to_le_bytes());
        buf[4..8].copy_from_slice(&self.memory_base.to_le_bytes());
        match self.initialised_at {
            Some(ts) => {
                buf[8] = 1;
                buf[9..17].copy_from_slice(&ts.to_le_bytes());
            }
            None => {} // already zeroed
        }
        buf[17..49].copy_from_slice(&self.module_hash);
        Cow::Owned(buf)
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        let schema_version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let memory_base = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let has_init = bytes[8];
        let init_val = u64::from_le_bytes(bytes[9..17].try_into().unwrap());
        let initialised_at = if has_init == 1 { Some(init_val) } else { None };
        let mut module_hash = [0u8; 32];
        module_hash.copy_from_slice(&bytes[17..49]);
        Self {
            schema_version,
            memory_base,
            initialised_at,
            module_hash,
        }
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 49,
        is_fixed_size: true,
    };
}

/// u64 wrapper for nonce storage. Little-endian encoding.
#[derive(Clone, Debug, Default)]
pub(crate) struct StorableU64(pub u64);

impl Storable for StorableU64 {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.0.to_le_bytes().to_vec())
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        Self(u64::from_le_bytes(bytes[0..8].try_into().unwrap()))
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 8,
        is_fixed_size: true,
    };
}

/// Optional timestamp for tombstoned_at: 1 byte discriminant + 8 bytes u64 (LE).
///
/// **This field is engine-owned.** Only `execute_deletion()` may set it.
/// External code must not write to this slot directly.
#[derive(Clone, Debug, Default)]
pub(crate) struct OptionalTimestamp(pub Option<u64>);

impl Storable for OptionalTimestamp {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut buf = vec![0u8; 9];
        if let Some(ts) = self.0 {
            buf[0] = 1;
            buf[1..9].copy_from_slice(&ts.to_le_bytes());
        }
        Cow::Owned(buf)
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        if bytes[0] == 1 {
            let ts = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
            Self(Some(ts))
        } else {
            Self(None)
        }
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 9,
        is_fixed_size: true,
    };
}

/// Receipt value wrapper (CBOR-encoded DeletionReceipt).
///
/// Max size: 8192 bytes. Current receipts are well under this limit.
/// If future fields push receipts larger, increase this bound.
#[derive(Clone, Debug)]
pub(crate) struct ReceiptBytes(pub Vec<u8>);

impl Storable for ReceiptBytes {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0)
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        Self(bytes.to_vec())
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 8192,
        is_fixed_size: false,
    };
}

// ---------------------------------------------------------------------------
// Main storage struct
// ---------------------------------------------------------------------------

pub(crate) struct MktdStorage {
    pub meta: StableCell<MetaCell, Memory>,
    pub state_hash: StableCell<Hash32, Memory>,
    pub nonce: StableCell<StorableU64, Memory>,
    pub certified_commitment: StableCell<Hash32, Memory>,
    pub deletion_event_hash: StableCell<Hash32, Memory>,
    pub manifest_hash: StableCell<Hash32, Memory>,
    pub receipts: StableBTreeMap<Hash32, ReceiptBytes, Memory>,
    pub tombstoned_at: StableCell<OptionalTimestamp, Memory>,
}

thread_local! {
    static STORAGE: RefCell<Option<MktdStorage>> = RefCell::new(None);
}

/// Schema version for the current storage layout.
const SCHEMA_VERSION: u32 = 1;

/// Initialise or reconnect to MKTd02 stable memory slots.
///
/// Called by both `init()` and `on_post_upgrade()`.
pub(crate) fn setup_storage(mm: &MemoryManager<DefaultMemoryImpl>, base: u8) {
    // Range check
    if (base as u16) + 7 > 255 {
        ic_cdk::trap(&format!(
            "MKTd02: base MemoryId {} + 7 exceeds 255. Choose a lower base.",
            base
        ));
    }

    let storage = MktdStorage {
        meta: StableCell::init(mm.get(MemoryId::new(base)), MetaCell::default())
            .expect("MKTd02: failed to init meta cell"),
        state_hash: StableCell::init(mm.get(MemoryId::new(base + 1)), Hash32::default())
            .expect("MKTd02: failed to init state_hash cell"),
        nonce: StableCell::init(mm.get(MemoryId::new(base + 2)), StorableU64::default())
            .expect("MKTd02: failed to init nonce cell"),
        certified_commitment: StableCell::init(
            mm.get(MemoryId::new(base + 3)),
            Hash32::default(),
        )
        .expect("MKTd02: failed to init certified_commitment cell"),
        deletion_event_hash: StableCell::init(
            mm.get(MemoryId::new(base + 4)),
            Hash32::default(),
        )
        .expect("MKTd02: failed to init deletion_event_hash cell"),
        manifest_hash: StableCell::init(mm.get(MemoryId::new(base + 5)), Hash32::default())
            .expect("MKTd02: failed to init manifest_hash cell"),
        receipts: StableBTreeMap::init(mm.get(MemoryId::new(base + 6))),
        tombstoned_at: StableCell::init(
            mm.get(MemoryId::new(base + 7)),
            OptionalTimestamp::default(),
        )
        .expect("MKTd02: failed to init tombstoned_at cell"),
    };

    // Collision detection (belt-and-suspenders):
    // Check initialised_at, schema_version, or memory_base to detect prior init.
    let existing = storage.meta.get();
    let previously_initialised = existing.initialised_at.is_some()
        || existing.schema_version != 0
        || existing.memory_base != 0;

    if previously_initialised && existing.memory_base != base as u32 {
        ic_cdk::trap(&format!(
            "MKTd02 already initialised at base={}; requested base={}",
            existing.memory_base, base
        ));
    }

    STORAGE.with(|s| {
        *s.borrow_mut() = Some(storage);
    });
}

/// Access storage immutably. Traps if not initialised.
pub(crate) fn with_storage<R>(f: impl FnOnce(&MktdStorage) -> R) -> R {
    STORAGE.with(|s| {
        let borrow = s.borrow();
        let storage = borrow
            .as_ref()
            .unwrap_or_else(|| ic_cdk::trap("MKTd02: not initialised. Call init() first."));
        f(storage)
    })
}

/// Access storage mutably. Traps if not initialised.
pub(crate) fn with_storage_mut<R>(f: impl FnOnce(&mut MktdStorage) -> R) -> R {
    STORAGE.with(|s| {
        let mut borrow = s.borrow_mut();
        let storage = borrow
            .as_mut()
            .unwrap_or_else(|| ic_cdk::trap("MKTd02: not initialised. Call init() first."));
        f(storage)
    })
}

/// Check whether storage has been set up (for is_initialised checks).
pub(crate) fn storage_exists() -> bool {
    STORAGE.with(|s| s.borrow().is_some())
}

pub(crate) const fn schema_version() -> u32 {
    SCHEMA_VERSION
}
