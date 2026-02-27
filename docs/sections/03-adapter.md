## Implement MKTdDataSource Adapter

The canister must implement the `MKTdDataSource` trait, which maps the library's generic interface to the canister's specific storage structure. In Leaf mode, the adapter exposes the canister's PII-bearing state as a single hashable unit. Typical adapter size: 30–80 lines of Rust.

### Required trait methods

| Method | Returns | Purpose |
|---|---|---|
| `mode()` | `CommitMode::Leaf` | Declares Leaf mode (single data subject per canister). |
| `pii_field_manifest()` | `Vec<FieldDescriptor>` | Enumerates PII fields with name, type, and `field_order`. This is the formal PII boundary definition. |
| `manifest_hash()` | `[u8; 32]` | Computes hash of manifest via `compute_manifest_hash()`. Freezes PII boundary for receipt verifiability. |
| `get_state_bytes()` | `Vec<u8>` | Serialises PII fields to deterministic CBOR bytes via `encode_pii_state()`. Input to state hash. |
| `tombstone_state()` | — | Replaces each PII field with hex-encoded `TOMBSTONE_CONSTANT`. Non-PII fields survive. |
| `is_tombstoned()` | `bool` | Returns `true` if all PII fields contain the tombstone constant. |

### State Encoding Spec requirement

The adapter must publish a State Encoding Spec documenting: field names, types, serialisation order, tombstone values, and encoding library version. This spec is anchored by `manifest_hash` — any change to the spec changes the manifest hash, ensuring receipt–spec alignment.

See the Deterministic Encoding Requirements section (below) for the full CBOR checklist and adapter correctness invariant.
