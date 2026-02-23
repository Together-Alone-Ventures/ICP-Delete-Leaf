# MKTd02 — Cryptographically Verifiable Deletion Receipts for ICP

A composable Rust library that any Internet Computer canister can import to produce **CVDRs** (Cryptographically Verifiable Deletion Receipts) for GDPR right-to-erasure compliance.

## Architecture

```
MKTd02/ (workspace)
├── zombie-core/     Pure Rust: hashing, types, receipt structures
├── mktd02/          Leaf-mode engine (ICP-dependent)
├── mktd02-macros/   #[mktd_guard] procedural macro
└── examples/        Reference adapters
```

### Crate Dependency Chain

- **zombie-core** → `sha2`, `ciborium`, `serde`, `candid` (no ICP deps)
- **mktd02** → `zombie-core`, `ic-cdk`, `ic-stable-structures`
- **mktd02-macros** → `syn`, `quote`, `proc-macro2`
- **Your canister** → `mktd02` (which re-exports `zombie-core` types)

## Status

🚧 **v0.1.0 — Under construction**

## License

Apache-2.0
