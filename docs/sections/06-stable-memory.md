## Stable Memory Coordination

MKTd02 reserves 8 contiguous stable memory slots starting from a configurable base MemoryId (default: 100). The enterprise canister must not allocate its own structures in the range `[base, base+7]`.

| Offset | Content | Type | Notes |
|---|---|---|---|
| base+0 | Meta cell | `u32, u32, Option<u64>, [u8;32]` | `schema_version`, `memory_base`, `initialised_at`, `module_hash`. Collision detection on init. |
| base+1 | state_hash | `[u8; 32]` | Current PII state hash. |
| base+2 | nonce | `u64` | Monotonic, never reused. |
| base+3 | certified_commitment | `[u8; 32]` | Published as certified variable via `set_certified_data()`. |
| base+4 | deletion_event_hash | `[u8; 32]` | Initially zero hash. Set during deletion. |
| base+5 | manifest_hash | `[u8; 32]` | Frozen PII boundary. Change triggers upgrade cascade. |
| base+6 | receipt store | `StableBTreeMap<[u8;32], Vec<u8>>` | Keyed by `receipt_id`. Max 8192 bytes per receipt. |
| base+7 | tombstoned_at | `Option<u64>` | Engine-owned. Only `execute_deletion()` may write this. |

### Safety mechanisms

- **Range check:** `base + 7` must be ≤ 255, else trap with descriptive error.
- **Collision detection (belt-and-suspenders):** If meta cell has `schema_version ≠ 0`, `memory_base ≠ 0`, or `initialised_at` set, and `memory_base` differs from requested base, trap.
- **Schema version gate:** On reconnect, if the stored `schema_version` does not match the expected version, trap with a descriptive error. This prevents silent data misinterpretation after layout changes. Downgrades are not supported.
- **Salt derivation:** The `mktd_salt` used in hash computations is NOT stored in stable memory. It is deterministically derived at runtime: `SHA-256(MKTD02_SALT_V1 || canister_id_bytes)`.

### Upgrade behaviour

On `post_upgrade`: if `manifest_hash` has changed (new PII field added/removed/renamed), the library executes a full recomputation cascade: update `manifest_hash` → recompute `state_hash` → recompute + publish `certified_commitment`. Old receipts remain verifiable under the `manifest_hash` recorded in each receipt. Module hash is updated unconditionally on every `post_upgrade`.

**Endianness convention:** Stable memory encoding uses little-endian; hash preimages use big-endian. These are separate domains.
