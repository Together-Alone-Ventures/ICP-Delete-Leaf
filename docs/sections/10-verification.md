## Verification

Independent verification tools for CVDRs produced by MKTd02 are maintained in a separate repository: [CVDR-Verify](https://github.com/Together-Alone-Ventures/CVDR-Verify).

This separation is deliberate — the verification repo contains zero deletion engine code, zero adapter code, and zero business logic. It verifies; it does not delete.

### Available tools

| Tool | Location | Scope |
|---|---|---|
| Shell script (`verify-quick.sh`) | `mktd02/verify-quick.sh` | V1 (partial), V3, V4 — quick smoke test via dfx |
| Rust CLI (`mktd02-verify`) | `mktd02/mktd02-verify/` | V1–V4 full verification including BLS certificate checking |

### Verification procedures

| Check | What it verifies |
|---|---|
| **V1** | Recomputes all cryptographic hashes from raw receipt fields. Confirms receipt integrity — no fields have been tampered with. |
| **V2** | Verifies the ICP subnet's BLS certificate and confirms the certified data matches the receipt's commitment. This is the trust anchor — it proves the subnet attested to the deletion. |
| **V3** | Compares the module hash in the receipt against the canister's current code on-chain. Three-way classification: Match, Mismatch-Expected (known upgrade), Mismatch-Suspicious. |
| **V4** | Confirms the tombstone is still active and the current state hash matches the receipt's post-deletion state hash. Proves the deletion is persistent. |

### What verification cannot check

No tool can verify that an enterprise's adapter correctly maps all PII fields to the manifest (Residual Trust assumption RT3). That requires source code audit of the adapter implementation. See the [Disclaimer](https://github.com/Together-Alone-Ventures/CVDR-Verify#disclaimer) in the CVDR-Verify repository.
