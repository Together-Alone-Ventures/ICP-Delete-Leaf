# MKTd02 — Cryptographically Verifiable Deletion Receipts for ICP

A composable Rust library that any Internet Computer canister can import to produce **CVDRs** (Cryptographically Verifiable Deletion Receipts) for GDPR right-to-erasure compliance.

## Quick Start

### 1. Add dependency

```toml
[dependencies]
mktd02 = { git = "https://github.com/Together-Alone-Ventures/MKTd02", package = "mktd02" }
```

### 2. Implement the adapter trait

```rust
impl MKTdDataSource for MyAdapter {
    fn mode(&self) -> CommitMode { CommitMode::Leaf }
    fn pii_field_manifest(&self) -> Vec<FieldDescriptor> { /* ... */ }
    fn manifest_hash(&self) -> [u8; 32] { compute_manifest_hash(&self.pii_field_manifest()) }
    fn get_state_bytes(&self) -> Vec<u8> { encode_pii_state(&my_pii).unwrap() }
    fn tombstone_state(&mut self) { /* write TOMBSTONE_CONSTANT to all PII fields */ }
    fn is_tombstoned(&self) -> bool { /* check all PII fields == TOMBSTONE_CONSTANT */ }
}
```

### 3. Wire into canister lifecycle

```rust
#[init]
fn init(owner: Principal) {
    // ... your init logic ...
    mktd02::init(&adapter, &memory_manager, MktdConfig::default());
}

#[post_upgrade]
fn post_upgrade() {
    // ... your upgrade logic ...
    mktd02::on_post_upgrade(&adapter, &memory_manager, config, module_hash);
}
```

### 4. Guard PII writes and refresh state hash

```rust
#[mktd_guard]
fn upsert_profile(input: ProfileInput) -> Result<(), MyError> {
    // ... write PII ...
    mktd02::refresh_state_hash(&adapter);
    Ok(())
}
```

### 5. Execute deletion

```rust
fn delete_profile() -> Result<(), MyError> {
    let receipt_id = mktd02::execute_deletion(&mut adapter, &config)?;
    Ok(())
}
```

## Architecture

```
MKTd02/ (workspace)
├── zombie-core/     Pure Rust: hashing, types, receipt structures (no ICP deps)
├── mktd02/          Leaf-mode CVDR engine (ICP-dependent)
├── mktd02-macros/   #[mktd_guard] procedural macro
└── examples/        Reference adapters (StableCell, StableBTreeMap, webhook)
```

### Dependency Chain

- **zombie-core** → sha2, ciborium, serde, candid
- **mktd02** → zombie-core, ic-cdk, ic-stable-structures
- **mktd02-macros** → syn, quote, proc-macro2
- **Your canister** → mktd02 (re-exports zombie-core types)

## Stable Memory Coordination

MKTd02 allocates **8 stable memory slots** starting from a configurable base MemoryId (default: 100).

| Offset  | Content              | Type                    |
|---------|----------------------|-------------------------|
| base+0  | Meta cell            | schema, base, init, mod |
| base+1  | state_hash           | [u8; 32]                |
| base+2  | nonce                | u64                     |
| base+3  | certified_commitment | [u8; 32]                |
| base+4  | deletion_event_hash  | [u8; 32]                |
| base+5  | manifest_hash        | [u8; 32]                |
| base+6  | receipt store        | StableBTreeMap          |
| base+7  | tombstoned_at        | Option<u64>             |

**Range check:** `base + 7` must be `<= 255`, validated on init.

**Collision detection:** If MKTd02 detects a prior initialisation with a different base, it traps with a descriptive error.

Your canister must not allocate stable structures in the range `[base, base+7]`.

## Guard Checklist

Two options for protecting PII-mutating functions:

| Approach | Returns | Behaviour |
|---|---|---|
| `#[mktd_guard]` macro | `Result<T, E: GuardError>` | Returns `Err` on violation |
| `mktd02::assert_can_write()` | `()` | Traps on violation |

Both check: (1) MKTd02 is initialised, (2) canister is not tombstoned.

## Naming Convention Table

| Name | Kind | Purpose |
|---|---|---|
| `MKTD_TOMBSTONE_V1` | Constant seed | SHA-256 seed for TOMBSTONE_CONSTANT (written to storage) |
| `MKTD02_TOMBSTONE_HASH_V1` | Domain tag | Tag for tombstone_hash in receipt |
| `MKTD02_EVENT_V1` | Domain tag | Tag for deletion_event_hash |
| `MKTD02_CERTIFIED_V1` | Domain tag | Tag for certified_commitment |
| `MKTD02_RECEIPT_V1` | Domain tag | Tag for receipt_id derivation |
| `MKTD02_SALT_V1` | Domain tag | Tag for per-canister salt |
| `MKTD02_MANIFEST_V1` | Domain tag | Tag for manifest_hash |

**Key distinction:** The tombstone constant is a *value written to storage*; domain tags are *prefixes for hash computations*.

## Module Hash Pipeline Rule

> **Hash what you ship.**

The module hash embedded in the receipt must be computed from the **final post-ic-wasm-shrink WASM bytes** — the exact bytes that get deployed. For local dev, zeros are accepted. For mainnet: two-pass build or pass hash via factory's `install_code` argument.

## Upgrade Behaviour

On every `post_upgrade`:

1. **Manifest check:** If `manifest_hash` changed → full recomputation cascade (state_hash → certified_commitment → publish)
2. **Module hash:** Updated unconditionally in meta cell

Old receipts remain verifiable under the `manifest_hash` recorded in each receipt.

## Canonicalisation Checklist

For adapters implementing `MKTdDataSource`:

- [ ] PII fields listed in `pii_field_manifest()` in ascending `field_order`
- [ ] `get_state_bytes()` uses `encode_pii_state()` (not raw ciborium)
- [ ] `tombstone_state()` writes `TOMBSTONE_CONSTANT` to **every** PII field
- [ ] `is_tombstoned()` checks **all** PII fields == `TOMBSTONE_CONSTANT`
- [ ] No `f32`/`f64` fields in PII state (floats are rejected)
- [ ] No `HashMap`/`BTreeMap` at the PII struct level (use structs)
- [ ] Field order in manifest matches field order in serialisation struct

## Status

**v0.1.0 — Under construction**

## License

Apache-2.0
