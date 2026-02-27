## Add MKTd02 Library Crate

Add the MKTd02 crates as dependencies in the canister's `Cargo.toml`:

```toml
[dependencies]
mktd02 = { git = "https://github.com/Together-Alone-Ventures/MKTd02.git", package = "mktd02" }
zombie-core = { git = "https://github.com/Together-Alone-Ventures/MKTd02.git", package = "zombie-core" }
hex = "0.4"  # For tombstone constant encoding
```

The crate provides: the core engine (state hashing, tombstone operations, receipt generation), the `MKTdDataSource` trait, the receipt export module, and certified variable management helpers.

**ic-cdk version requirement:** MKTd02 depends on ic-cdk 0.17. If the canister uses an earlier version (e.g., 0.15 or 0.16), it must be bumped. This may affect other canisters in the same workspace.
