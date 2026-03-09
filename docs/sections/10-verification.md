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

### V2 model (pending vs finalized)

The reference verifier provides V2 verification paths for MKTd02 receipts.
Current behavior is intentionally strict:

- Pending receipts (no embedded certificate yet) use a live certified-query path as secondary corroboration/fallback.
- Finalized receipts are expected to carry `bls_certificate` and `trust_root_key_id` and use the embedded-certificate path as the primary long-term evidentiary route.
- In archived receipt-contained verification, only freshness-at-verification-time is intentionally relaxed.
- Signature authenticity, delegation trust, canister authorization, and certified-data commitment matching are still verified.
- `trust_root_key_id` is validated against known key metadata.
- Operational/key-rotation behavior is bounded by current verifier and agent capabilities; documented limitations should be treated as active constraints.

### V3 model (archival-first)

- Primary path: archival provenance (`module_hash` in receipt -> published build/release record such as `RELEASES` -> reproducible build -> inspectable source).
- Secondary path: live on-chain module-hash corroboration, where infrastructure still exists.
- `module_hash` is the SHA-256 of the exact deployed WASM bytes. It is not a special ICP object with extra canister metadata.

### Export/anchoring implications

- Finalized receipt export is meaningful evidentiary preservation, not only convenience backup.
- V1 and V2 can be verified from the exported finalized receipt artifact alone.
- V3 additionally requires published release/build provenance.
- V4 remains a live canister/state check.

### Deterministic CBOR note

Deterministic encoding statements are project-rule-specific and should not be interpreted as universal RFC canonical-CBOR equivalence claims.

### Scope boundary

Verifier outputs validate cryptographic/consistency properties of receipts and related on-chain evidence.
They do not by themselves prove completeness of product-specific PII field mapping without adapter/source review.
