# MKTd02 — Cryptographically Verifiable Deletion Receipts for ICP

MKTd02 is a Rust library for Internet Computer canisters that produces CVDRs (Cryptographically Verifiable Deletion Receipts) for Leaf-mode canisters (single data subject per canister).

## Scope

This README is intentionally concise.
Detailed and normative wording lives in:

- `docs/sections/03-adapter.md`
- `docs/sections/06-stable-memory.md`
- `docs/sections/10-verification.md`
- `docs/architecture/finalization-flow.md`

## Key Invariants (v0.3.x)

- Receipt ID derivation:
  - `receipt_id = hash_with_tag(TAG_RECEIPT_V3, [u32_be(len(canister_id_bytes)), canister_id_bytes, u32_be(len(record_id_bytes)), record_id_bytes, u64_be(deletion_seq)])`
- Three-phase receipt lifecycle:
  - Phase A (update): tombstone + pending receipt
  - Phase B (ingress query): capture certificate material from query context
  - Phase C (update): finalize pending receipt by embedding certificate fields
- MKTd02 Leaf-mode receipts are `mktd02-v3` and include `record_id` and `deletion_seq`.
- `subnet_id` is no longer part of `MktdConfig` or `DeletionReceipt`.
- `manifest_hash` and `commit_mode` are not part of Leaf-mode receipt fields.

## Leaf-Mode `record_id` rule

`record_id` is an opaque byte vector at schema level.
In MKTd02 leaf mode specifically, it is derived internally as `ic_cdk::caller().as_slice().to_vec()`.
This default rule assumes the deletion/tombstone call is invoked directly on the subject canister by the authenticated data subject principal.
If your integration uses an intermediary/orchestrator canister for c2c deletion calls, review and adapt `record_id` derivation for that architecture.

## ICP Platform Constraint (A→B→C)

The A→B→C pattern is driven by ICP semantics: `ic0.data_certificate()` is query-only.
This is a platform constraint, not product-specific behavior.

## Finalization Identity Invariant

Phase A persists the pending `receipt_id` while finalization lock is held.
Phase B and Phase C read that persisted pending identity directly; they do not recompute from live context.

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

## Updating the Integration Guide

`MKTd02_Integration_Guide.md` at the repo root is a **generated file**.
Do not edit it directly — changes will be overwritten on the next compose run.

**Source files:**
- Section content lives in `docs/sections/` (local) and
  `zombie-delete-docs` (shared platform content, pinned in `docs/compose.yaml`)
- Section order and structure is defined in `docs/compose.yaml`

**To rebuild after changing any section file:**
```bash
# Requires a local clone of zombie-delete-docs at the pinned ref
git clone --depth 1 --branch v1.0.3 \
  https://github.com/Together-Alone-Ventures/zombie-delete-docs.git \
  /tmp/shared-docs

pip install pyyaml
python3 docs/scripts/compose.py /tmp/shared-docs docs/compose.yaml

# Commit the result
git add MKTd02_Integration_Guide.md
git commit -m "docs: recompose Integration Guide"
git push
```

The GitHub Action (`.github/workflows/build-guide.yml`) also runs the
compose step on release and produces a downloadable PDF artifact, but
does not commit the output — the committed markdown is the canonical
repo artifact.

## Status

Current protocol line in this repository: v0.3.x (Leaf mode).
Treat v0.2.0-era formulas/field lists as historical unless explicitly marked otherwise.
