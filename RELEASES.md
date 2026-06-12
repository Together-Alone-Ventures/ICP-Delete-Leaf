# MKTd02 — Release Provenance (repo: Together-Alone-Ventures/ICP-Delete-Leaf)

Records release provenance for the `mktd02` library crate, supporting V3 (Canister Module Verification).

**Provenance model.** `mktd02` is a plain Rust library crate and produces no standalone WASM. The module hash used by V3 is the SHA-256 of the *deployed integrator/reference canister's* WASM bytes (stamped into each CVDR), reproduced by rebuilding that canister with the library reproducibility inputs pinned below. A library release anchors to **reproducibility inputs**, not a library WASM hash. Deployment/reference-canister module hashes are recorded per integration, not at the library level.

## Current line

`mktd02-v0.4.1` (Leaf mode; dependency re-pin of v0.4.0). Supersedes the v0.3.x line; the earlier v0.2.x line is historical.

## mktd02-v0.4.1 — library reproducibility inputs

| Field | Value |
|---|---|
| Repo | Together-Alone-Ventures/ICP-Delete-Leaf |
| Tag | mktd02-v0.4.1 (annotated; GitHub Release published) |
| Commit | the commit mktd02-v0.4.1 points to (resolve via `git rev-parse mktd02-v0.4.1`) |
| Crate versions | mktd02 0.4.1, mktd02-macros 0.4.1 (workspace-inherited, lockstep) |
| zombie-core dependency | tag zombie-core-v0.3.1, commit 508f2f8bb88f4395293168c6ef25c92a67dee894 (unchanged from v0.4.0) |
| Rust toolchain | rustc 1.90.0 (1159e78c4 2025-09-14); cargo 1.90.0 (840b83a10 2025-07-30); stable-x86_64-unknown-linux-gnu |
| Feature posture | default features (default = []); local-replica OFF (must never be enabled in release/prod) |
| Deployment / reference-canister module hash | not applicable at library level — see Reference integration artifact |

Scope: dependency re-pin only — ic-cdk 0.17.2 -> 0.18.x, ic-stable-structures 0.6.9 -> 0.7.2. Patch-class under the 1d test: public Rust API, receipt schema, hash formulas, and the 8-slot stable-memory layout (100-107) unchanged; on-disk encodings byte-compatible for the bound shapes in use. E4-lite signed off (G); independent reviews: Claude + CD (CD re-ran full build/test green). Evidence: MKTd02_v0.4.1_RePin_Deliverable.md.

Build / test commands run, with results:

- `cargo build --locked` — pass
- `cargo test -p mktd02 --locked` — 9 passed / 0 failed (mktd02 lib unit tests)
- `cargo build --locked --target wasm32-unknown-unknown` — clean
- PocketIC harness (harness/, separate workspace, own lock) — 3/3 passed: A1 wrapper parity, A2 host-supplied record_id, two-cell controller guard

Integration Guide regeneration: publishing the GitHub Release triggers .github/workflows/build-guide.yml, which regenerates and attaches MKTd02_Integration_Guide.md (the permissions fix is on this commit line, so the attach is expected to succeed).

## mktd02-v0.4.0 — library reproducibility inputs

| Field | Value |
|---|---|
| Repo | Together-Alone-Ventures/ICP-Delete-Leaf |
| Tag | mktd02-v0.4.0 (annotated; GitHub Release published) |
| Commit | the commit mktd02-v0.4.0 points to (resolve via `git rev-parse mktd02-v0.4.0`); built on fdd19b0 |
| Crate versions | mktd02 0.4.0, mktd02-macros 0.4.0 (workspace-inherited, lockstep) |
| zombie-core dependency | tag zombie-core-v0.3.1, commit 508f2f8bb88f4395293168c6ef25c92a67dee894 (resolves to crate version 0.3.0 — known zombie-core tag/version note) |
| Rust toolchain | rustc 1.90.0 (1159e78c4 2025-09-14); cargo 1.90.0 (840b83a10 2025-07-30); stable-x86_64-unknown-linux-gnu |
| Feature posture | default features (default = []); local-replica OFF (must never be enabled in release/prod) |
| Deployment / reference-canister module hash | not applicable at library level — see Reference integration artifact |

Build / test commands run, with results:

- `cargo build --locked` — pass
- `cargo test` — 9 passed / 0 failed (mktd02 lib unit tests); mktd02-macros 0 tests; 0 doc-tests
- `cargo clippy` — 3 warnings, all pre-existing style lints in mktd02/src/storage.rs (not modified by P0); none introduced by the P0 patch
- `cargo build --locked --target wasm32-unknown-unknown` — clean (library compiles for the canister target)
- PocketIC harness (harness/, separate workspace, own lock) — 3/3 passed in P0 validation: A1 wrapper parity, A1 host-supplied, two-cell controller guard

Integration Guide regeneration: publishing the GitHub Release triggers .github/workflows/build-guide.yml, which regenerates MKTd02_Integration_Guide.md from the compose.yaml default zombie-delete-docs ref. Any resulting guide diff is a release-note observation, not a build failure.

## Reference integration artifact — DaffyDefs (pending)

To be completed after the DaffyDefs bump to mktd02-v0.4.1 and build-compat regression (R2), which produce a deployed reference-canister module hash. (The v0.4.0 reference artifact was never recorded; superseded by v0.4.1.)

| Field | Value |
|---|---|
| Reference canister | DaffyDefs profile_canister |
| Deployed module hash | to be recorded after regression |
| Build toolchain / flags | to be recorded after regression |

## Historical entries (provenance not recorded at release time)

Cut before a provenance record was maintained. Factual items backfilled; toolchain hashes, WASM/module hashes, Arweave IDs, and CI status were not recorded at release time and are not reconstructed here.

| Tag | Commit | Date | Status |
|---|---|---|---|
| mktd02-v0.3.1 | 7d9584c2e62161f7a4df7e4bf1ff70bb544fbbe2 | 2026-03-12 | Historical — provenance not recorded |
| mktd02-v0.3.0 | 90e4234382fc20a2eed6172c09bec5a42d560a3f | 2026-03-11 | Historical — provenance not recorded |
| v0.2.x | (doc/protocol line) | — | Historical documentation line, superseded |
| v0.1.0 | (historical) | — | Historical reference; not current protocol/formula authority |

## Release-infrastructure debt (tracked; pre-existing, not a P0 blocker)

The release ceremony historically described for this repo is not implemented as infrastructure. Acknowledged as pre-existing debt to address outside P0:

1. Release CI / local-replica build guard — mktd02/Cargo.toml references .github/workflows/release.yml, which does not exist; the local-replica-off guarantee is not CI-enforced.
2. Arweave archival automation — referenced in working notes; absent from this repo.
3. RELEASES schema definition — formalize the library-reproducibility vs deployment-module-hash split used here.
4. Stale Cargo comment — remove/fix the release.yml reference in mktd02/Cargo.toml.
5. Vestigial mktd02-macros member — no dependency edge from mktd02; confirm intended use or remove.

## Verification reminder

V3 verifies the deployed canister's module hash against the exact bytes install_code received (after all transforms — ic-wasm shrink, gzip, etc.), reproduced via the reproducibility inputs above. The library has no standalone WASM to hash.

## Gate record

G endorses release path (a): tag v0.4.0 after honest RELEASES.md correction and explicit tech-debt logging. Missing release CI/Arweave automation is acknowledged as pre-existing release-infrastructure debt, not a retroactive P0 blocker. Library release provenance anchors to reproducibility inputs; deployed module_hash remains an integration/reference-canister artifact, not a library WASM hash.
