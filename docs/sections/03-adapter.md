## Implement `MKTdDataSource` Adapter

The host canister implements `MKTdDataSource` to map product storage to MKTd02’s Leaf-mode interface.

### Leaf-Mode Boundary

MKTd02 is for single-subject-per-canister architecture (Leaf mode).
Multi-subject-per-canister architecture is out of scope for MKTd02 and belongs to Tree-mode product lines.

### Required trait methods

| Method | Returns | Purpose |
|---|---|---|
| `mode()` | `CommitMode` (use `CommitMode::Leaf` for MKTd02) | Declares integration mode. |
| `pii_field_manifest()` | `Vec<FieldDescriptor>` | Declares PII boundary metadata and ordering intent. |
| `get_state_bytes()` | `Vec<u8>` | Deterministic encoded state bytes used for hashing. |
| `tombstone_state()` | — | Applies tombstone writes to PII fields. |
| `is_tombstoned()` | `bool` | Post-condition check for tombstone state. |


### State Encoding Specification Requirement

Integrators should publish a State Encoding Spec documenting:

- Field names
- Field types
- Serialization order assumptions
- Tombstone value mapping
- Encoding library/version assumptions

This supports independent audit/reproduction of state-hash behavior.

### Deterministic CBOR Clarification

Determinism here is defined by project encoding constraints and integration rules.
It is not a blanket claim that arbitrary CBOR encoder output is interchangeable under RFC canonical-CBOR semantics.

### Adapter Correctness Invariant

After `tombstone_state()`, `is_tombstoned()` must reflect the expected post-condition before receipt finalization proceeds.
