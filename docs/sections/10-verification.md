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
| **V1** | Recomputes all cryptographic hashes from raw receipt fields
