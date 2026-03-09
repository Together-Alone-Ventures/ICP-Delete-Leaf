# MKTd02 Release Provenance

This file records release build provenance used for module-hash verification workflows.

## Current line (minimal status)

Primary maintained protocol/documentation line: v0.2.x (Leaf mode).

## Release entries

### v0.2.x (to be completed at tag cut)

| Field | Value |
|---|---|
| Version | v0.2.x |
| Commit | _to be filled at tag time_ |
| Rust toolchain | _pin exact version at tag time_ |
| Build command | `cargo build --release --target wasm32-unknown-unknown` |
| Post-build transform | `ic-wasm shrink` |
| WASM SHA-256 | _to be filled at tag time_ |

### v0.1.0 (historical)

v0.1.0 provenance remains historical reference and should not be treated as current protocol/formula authority.

## Verification reminder

Compare hashes against the exact shipped WASM bytes.
