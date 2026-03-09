# MKTd02 Finalization Flow (A → B → C)

## 1. Why this flow exists

The A→B→C flow is shaped by ICP platform semantics:

- Phase A (delete) runs in update context.
- `ic0.data_certificate()` is query-only.
- Finalization then runs in update context.

So “seamless” means no manual operator step is required in normal operation, not “single canister call.”

## 2. Canonical phase sequence

### Phase A — Update
- Apply tombstone writes
- Emit/store pending receipt
- Acquire finalization lock

### Phase B — Ingress Query
- Capture certificate material from query context
- Return material needed to continue orchestration

### Phase C — Update
- Finalize pending receipt by embedding finalization fields
- Release finalization lock

## 3. Authorization wording (clarified)

At the library layer, finalization on the target canister is controller-guarded.

Integration architecture may vary. One allowed pattern is a constrained proxy/orchestrator path that validates inputs and forwards finalization through an authorized route.
This is an allowed pattern, not a mandated default architecture.

## 4. Idempotency and interruption handling

Integrations should handle interruption between phases and resume safely.
“Already finalized” should be handled as an idempotent terminal condition where appropriate.

## 5. Script boundary (explicit)

- Generic MKTd02 recovery/finalization tooling lives in:
  - `MKTd02/scripts/finalize_receipt_generic.sh`
- Product-specific operational wrappers should live in product repositories.

This architecture note is protocol/integration guidance; product wrappers are examples, not canonical protocol tooling.
