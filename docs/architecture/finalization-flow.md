# MKTd02 v0.2.0 — Seamless, Fully Automated Deletion & CVDR Finalization
**Architecture Note v2 | Together Alone Ventures | March 2026**

## 1. Executive Summary
MKTd02 delivers a fully automated deletion workflow: a user clicks Delete; the record is tombstoned, a **Pending** receipt is created immediately, and the receipt is automatically finalized — including the embedded BLS certificate required for future-proof offline verification — **without any manual admin action or timer**.

This note explains the design, why it is shaped the way it is, and what enterprise integrators must do to implement it correctly.

## 2. Platform Constraint That Determines the Design
ICP’s certificate model imposes one hard constraint: **`ic0.data_certificate()` is query-only**.

Therefore:
- Deletion/tombstoning must occur in an **update** call.
- The BLS certificate capture must occur in an **ingress query**.
- “Seamless” cannot mean “single canister call”; it means **no human/admin steps** after the user clicks Delete.

## 3. Three-Phase Automated Workflow (A → B → C)
The three phases run back-to-back within a single frontend async function. From the user’s perspective it is one action.

### Phase A — Update (Delete + Pending Receipt + Finalization Lock)
Triggered by the user’s Delete action.

Inside the target canister:
- Perform the deterministic delete/tombstone procedure via the MKTd02-guarded write path.
- Compute and store receipt fields available during update: pre/post state hashes, `deletion_event_hash`, `certified_commitment`, `module_hash`, `nonce`, `timestamp`.
- Set receipt status = **Pending**.
- Acquire the **finalization lock**: prevents upgrades or certified-data changes that would invalidate the pending certificate linkage (any call that would mutate certified data traps while the lock is held).

User-visible outcome: Delete returns success immediately. A Pending receipt exists right away. The BLS certificate will be embedded within seconds.

### Phase B — Ingress Query (Capture the BLS Certificate Blob)
Automatically triggered by the frontend immediately after Phase A returns.

Frontend calls `mktd_get_certificate()` as an ingress query:
- The runtime populates `ic0.data_certificate()` in query context.
- The canister returns `(receipt_id, certified_commitment, certificate_blob)`.

Returning all three values allows the orchestrator to confirm which receipt and commitment the certificate covers.

This step MUST be an ingress query from an external caller. A canister cannot call `ic0.data_certificate()` from within an update, and inter-canister calls do not receive the platform certificate. The frontend is the correct orchestrator.

### Phase C — Update (Finalize Receipt + Release Lock)
Automatically triggered by the frontend immediately after Phase B succeeds.

Frontend calls the finalization endpoint via a controller-authorized path (e.g., a factory proxy in DaffyDefs):
- The proxy, as controller, calls into the target canister to finalize the pending receipt.
- The canister embeds the BLS certificate, stamps `trust_root_key_id` (NNS root key identifier), and sets receipt status = **Finalized**.
- Finalization lock is released.

Outcome: Within seconds of the user clicking Delete, the receipt is Finalized with its embedded certificate. The CVDR is fully verification-ready offline — V1 through V4 pass.

## 4. Where the Automation Lives
The orchestration of B → C must originate from ingress (frontend or off-chain agent). This is a platform requirement:
- A canister cannot fetch its own data certificate inside an update.
- A factory canister cannot obtain `ic0.data_certificate()` semantics via an inter-canister call.

Correct pattern:
- Frontend orchestrates B → C immediately after A
- plus recovery logic for interruptions.

In DaffyDefs, the delete handler calls all three phases in a single async function. Enterprise integrators must implement the same pattern in their own frontend or server-side agent.

Phase C requires a controller-authorized path. Integrators without a factory pattern must ensure their finalizing caller holds controller authority over the target canister.

## 5. Receipt Status State Machine
Receipts follow a strict state machine (no silent failure):
- **Pending:** Phase A complete, lock held, BLS cert not embedded.
- **Finalized:** Phase C complete, BLS cert embedded, lock released; receipt self-contained for offline V1–V4 verification.
- **Aborted (planned):** Controller escape hatch if finalization cannot complete; releases lock, records reason, preserves transparency. V2 offline verification unavailable for aborted receipts (live query fallback remains while canister exists).

Aborted is not implemented in v0.2.0; current escape hatch is manual recovery via the reference script (Section 8). A formal `abort()` endpoint is planned.

## 6. Trust Anchor and Key Rotation
Every finalized receipt carries `trust_root_key_id` — a string identifier for the NNS root public key used at finalization time (e.g. “mainnet”). This is embedded at Phase C by the MKTd02 library automatically.

This means:
- Receipts are self-describing with respect to the root key.
- After future key rotation, historical receipts remain verifiable by looking up `trust_root_key_id` in the allowlist.
- CVDR-Verify validates `trust_root_key_id` against a versioned allowlist before attempting BLS verification; unknown IDs fail closed.

## 7. Residual Risks and Mitigations
Primary risk: the process is interrupted between Phase A and Phase C (e.g., user closes tab).
Mitigations:
- Recovery-on-load: on app start, check `mktd_is_pending()` and auto-complete B → C if pending.
- Retry/backoff for transient Phase B/C failures.
- Treat “already finalized” as success (idempotency behavior).

## 8. Reference Recovery Script
A reference recovery script is provided for the failure case where Phase A completed but Phases B and C did not. This is an admin/operator tool, not part of the normal user flow.

DaffyDefs location (reference): `scripts/finalize_receipt.sh` :contentReference[oaicite:2]{index=2}  
Usage: `./scripts/finalize_receipt.sh <profile_canister_id>`

It checks `mktd_is_pending()`, runs Phase B (`mktd_get_certificate()`), runs Phase C (factory `finalize_profile_receipt()`), and confirms finalization.

Enterprise integrators should implement an equivalent recovery tool appropriate to their architecture. The DaffyDefs script is a reference, not a drop-in for other deployments.

## 9. Enterprise Integration Checklist
Integrators must:
- Implement all three phases in a single async delete handler (frontend or server-side agent).
- Implement recovery-on-load: check pending on app load and auto-complete B → C if pending.
- Ensure Phase C is called via a controller-authorized path.
- Handle idempotent finalization (“already finalized” = success).
- Implement retry logic with backoff for transient failures.
- Implement an admin recovery procedure (equivalent to the script).
- Do NOT embed TRUST_ROOT_KEY bytes in finalization — the library sets `trust_root_key_id` automatically.

## 10. Document Location and Updates
This document is maintained in the MKTd02 repository at:
`docs/architecture/finalization-flow.md`

Repository is authoritative; copies may be out of date.
