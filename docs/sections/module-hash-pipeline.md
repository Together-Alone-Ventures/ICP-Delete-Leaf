## Module Hash: Deployment Patterns

The module hash is the SHA-256 of the canister's deployed WASM binary — the exact bytes running on the subnet. Zombie Delete libraries stamp this value into every Cryptographically Verifiable Deletion Receipt (CVDR) as the `module_hash` field. It enables V3 (Canister Module Verification): a verifier can independently confirm the canister was running the expected code at deletion time. If the module hash is zeros, V3 cannot be performed.

**Core rule: Hash what you ship.** The SHA-256 must be computed from the final, post-transformation WASM bytes — the exact bytes that `install_code` receives. Not pre-shrink. Not pre-optimisation. The shipped bytes.

### Why the hash must come from outside the canister

A canister on ICP cannot read its own WASM hash at runtime — there is no system API equivalent to `ic0.self_module_hash()`. The hash must be provided by the entity that performs the deployment. Attempting to embed the hash at compile time creates a circular dependency: embedding it changes the WASM, which changes the hash. The solution is to always pass the hash as an init or upgrade argument from the deploying entity, which already holds the bytes.

### Deployment patterns

Every canister installation or upgrade on ICP flows through one chokepoint: the management canister's `install_code`. Whoever calls `install_code` already has the WASM bytes. The only variable is who makes that call and how the hash reaches the canister. The following table covers the complete space:

| Who calls `install_code` | Hash injection method | Notes |
|---|---|---|
| **Developer / ops (`dfx`)** | Init/upgrade arg from build script | `sha256sum` after all WASM transformations, pass as Candid arg. |
| **Internal deploy service (`ic-agent` library)** | Init/upgrade arg from deploy tool | Enterprise admin tooling that wraps `ic-agent` directly. Same principle as `dfx`. |
| **CI/CD pipeline** | Init/upgrade arg from pipeline step | Automated: build -> shrink -> hash -> deploy in one pipeline. |
| **Factory canister** | Factory computes hash at WASM upload, passes in child's init/upgrade arg | Factory already holds the bytes. Compute once at WASM upload time, store alongside the blob. |
| **Orchestrator canister (fleet)** | Same as factory — one hash computation, N upgrade calls | Enterprise fleet rollout pattern. Hash computed once, passed to all child canisters. |
| **Governance (SNS / DAO)** | Upgrade arg in proposal payload | If your governance path supports passing upgrade args, include the hash. Otherwise, use two-pass attestation (below) immediately after the governance-initiated upgrade. |
| **Self-upgrading canister** | Canister computes hash from fetched WASM before calling `install_code` on itself | The canister has the bytes in hand and can compute SHA-256 before triggering the upgrade. |
| **Blackholed (immutable)** | Set once at final deploy; never changes | Strongest V3 story. The module hash is permanent — code cannot be swapped. |

### Fallback: Two-pass attestation

If your deployment path genuinely cannot plumb the module hash into the init or upgrade argument on first deploy, use the two-pass pattern:

1. Deploy with `[0u8; 32]` as a placeholder.
2. Read the canister's module hash from `canister_status` (or `dfx canister info`) — this is the network's ground truth and resolves all ambiguity about shrink steps, gzip, or build transformations.
3. Immediately upgrade the canister with the correct hash as the upgrade argument.

**Note:** Any receipts generated in the window between first deploy and the corrective upgrade will contain zeros and will fail V3 verification. Minimise this window.

### Byte-identity warnings

- **Hash after all WASM transformations** (`ic-wasm shrink`, `wasm-opt`, gzip, etc.), not before. The canonical rule is: hash the exact bytes `install_code` receives.
- **When in doubt, use two-pass attestation.** Read `canister_status.module_hash` after deployment. The network's view is authoritative and resolves all ambiguity about which bytes were actually deployed.
- **Non-deterministic builds:** Same source code does not guarantee the same module hash across different machines or toolchain versions. If V3 reproducibility matters for auditors, pin your Rust toolchain version and build flags in `RELEASES`.

### Warning: `reinstall` mode

> ⚠ **ICP's `reinstall` mode wipes stable memory.** This destroys all Zombie Delete state including stored receipts, tombstone status, and deletion-sequence history. Treat `reinstall` as new genesis — prior CVDRs are irrecoverably lost. If your deployment process uses `reinstall` for any reason, be aware that the canister's entire deletion history is erased. Use `upgrade` mode for all routine deployments.
