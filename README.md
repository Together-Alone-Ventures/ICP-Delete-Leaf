# MKTd02 — Cryptographically Verifiable Deletion Receipts for ICP

MKTd02 is a Rust library for Internet Computer canisters that produces CVDRs (Cryptographically Verifiable Deletion Receipts) for Leaf-mode canisters (single data subject per canister).

## Scope

This README is intentionally concise.
Detailed and normative wording lives in:

- `docs/sections/03-adapter.md`
- `docs/sections/06-stable-memory.md`
- `docs/sections/10-verification.md`
- `docs/architecture/finalization-flow.md`

## Key Invariants (v0.2.x)

- Receipt ID derivation:
  - `receipt_id = hash_with_tag(TAG_RECEIPT, canister_id || nonce)`
- Three-phase receipt lifecycle:
  - Phase A (update): tombstone + pending receipt
  - Phase B (ingress query): capture certificate material from query context
  - Phase C (update): finalize pending receipt by embedding certificate fields
- v0.2.x Leaf-mode receipts do not include `manifest_hash` or `commit_mode`.

## ICP Platform Constraint (A→B→C)

The A→B→C pattern is driven by ICP semantics: `ic0.data_certificate()` is query-only.
This is a platform constraint, not product-specific behavior.

## Verification Status (Reference Verifier)

The reference verifier in `CVDR-Verify` provides V1–V4 verification paths.

V2 and V3 should be read as dual-path models:
- V2 primary (finalized receipts): embedded-certificate receipt-contained verification.
- V2 secondary (pending or still-live contexts): live certified-query corroboration path.
- V3 primary: archival module-hash provenance (`module_hash` -> published build/release record -> reproducible build -> inspectable source).
- V3 secondary: live on-chain module-hash corroboration where infrastructure still exists.

Finalized receipt export is evidentiary preservation:
- V1 and V2 can be verified from exported finalized receipt artifacts.
- V3 additionally requires published build/release provenance.
- V4 remains a live canister/state check.

See `docs/sections/10-verification.md` for precise wording.

## Deterministic CBOR Clarification

MKTd02 relies on deterministic encoding rules under project encoder constraints (`ciborium` + `serde` usage model).
This should not be read as a blanket RFC 8949 canonical-CBOR equivalence claim.

## Tooling Boundary

- Generic MKTd02 recovery/finalization tooling:
  - `scripts/finalize_receipt_generic.sh`
- Product-specific wrappers belong in product repositories.

## Related Repositories

- `CVDR-Verify` — verifier/reference verification tooling
- `DaffyDefs` — worked example/template integration

## Status

Current protocol line in this repository: v0.2.x (Leaf mode).
Treat v0.1.0-era formulas/field lists as historical unless explicitly marked otherwise.
