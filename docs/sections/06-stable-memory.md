## Stable Memory Coordination

MKTd02 reserves 8 contiguous memory slots from a configurable base (`base_memory_id`, default `100`).

| Offset | Content | Type | Notes |
|---|---|---|---|
| base+0 | meta | metadata cell | schema/base/init/module-hash fields |
| base+1 | state_hash | `[u8; 32]` | current state hash |
| base+2 | deletion_seq | `u64` | monotonic counter (same slot formerly labelled `nonce`) |
| base+3 | certified_commitment | `[u8; 32]` | certified commitment value |
| base+4 | deletion_event_hash | `[u8; 32]` | deletion-event hash value |
| base+5 | finalization_lock | `bool` | prevents certified-data drift between Phase A and C |
| base+6 | receipt store | `StableBTreeMap<[u8;32], Vec<u8>>` | CBOR-encoded receipts |
| base+7 | tombstoned_at | `Option<u64>` | tombstone timestamp cell |

### Safety checks

- Range check on base allocation
- Storage/base collision checks
- Schema/version compatibility checks on reconnect paths

### Upgrade/finalization interaction

When finalization lock is held (pending receipt), operations that would drift certified-data linkage are blocked by implementation guards.

### Clarification

Legacy `manifest_hash` terminology is not a v0.3 stable-memory slot in MKTd02.
The `base+2` slot position is unchanged in v0.3; only the semantic label changed from `nonce` to `deletion_seq`.
