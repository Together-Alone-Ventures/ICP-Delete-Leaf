## Verification

Independent verifier tooling is maintained in `CVDR-Verify`.
This separation is intentional: verifier code is independent from deletion/integration engine code.

### V1–V4 at a glance

| Check | Scope |
|---|---|
| V1 | Receipt hash-consistency checks (protocol formulas) |
| V2 | ICP certificate/BLS-path checks and certified-data match checks |
| V3 | Module-hash provenance checks |
| V4 | Tombstone/state-consistency checks |

### V2/BLS status (precise wording)

The reference verifier provides V2 verification paths for MKTd02 receipts.
Current behavior is intentionally strict:

- Finalized receipts are expected to carry `bls_certificate` and `trust_root_key_id`.
- `trust_root_key_id` is validated against known key metadata.
- Operational/key-rotation behavior is bounded by current verifier and agent capabilities; documented limitations should be treated as active constraints.

### Pending vs finalized context

- Pending receipts may require live-query certificate path.
- Finalized receipts may use embedded-certificate path.

### Deterministic CBOR note

Deterministic encoding statements are project-rule-specific and should not be interpreted as universal RFC canonical-CBOR equivalence claims.

### Scope boundary

Verifier outputs validate cryptographic/consistency properties of receipts and related on-chain evidence.
They do not by themselves prove completeness of product-specific PII field mapping without adapter/source review.
