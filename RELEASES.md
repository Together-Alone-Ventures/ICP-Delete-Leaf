# MKTd02 Release Provenance

Each entry records the build provenance for a tagged release. Verifiers can use this to close the V3 verification loop: compare the module hash from `dfx canister info` against the WASM SHA-256 listed here.

For verification tooling, see [CVDR-Verify](https://github.com/Together-Alone-Ventures/CVDR-Verify).

## v0.1.0

| Field | Value |
|---|---|
| **Version** | 0.1.0 |
| **Commit** | _to be filled at tag time_ |
| **Rust toolchain** | stable (specify exact version at tag time, e.g. 1.78.0) |
| **Build command** | `cargo build --release --target wasm32-unknown-unknown` |
| **Post-build** | `ic-wasm shrink` |
| **WASM SHA-256** | _to be filled at tag time_ |

> **Note:** Non-deterministic builds mean the same source may produce different WASM hashes on different machines or toolchain versions. If V3 reproducibility matters for your auditors, pin the exact Rust toolchain version and build on a clean environment.

## How to verify
```bash
# 1. Build from source at the tagged commit
git checkout v0.1.0
cargo build --release --target wasm32-unknown-unknown
ic-wasm target/wasm32-unknown-unknown/release/<crate>.wasm -o output.wasm shrink

# 2. Compare hash
sha256sum output.wasm
# Should match the WASM SHA-256 above
```
