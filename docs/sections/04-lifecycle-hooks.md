## Wire Lifecycle Hooks
Three integration points in the canister's lifecycle, plus a deployment-time module hash pipeline (covered in the next section).
### init()
Call `mktd02::init()` at the end of your canister's init function, after all initial data writes:
```rust
let adapter = MyAdapter;
let module_hash = [0u8; 32]; // ⚠ DEV ONLY — see Module Hash: Deployment Patterns for production
MEMORY_MANAGER.with(|mm| {
    mktd02::init(&adapter, &mm.borrow(), MktdConfig {
        base_memory_id: 100,
        subnet_id: Principal::from_text("jtdsg-3h6gi-...").unwrap(),
    }, module_hash);
});
```
**Subnet ID:** This value cannot be discovered at runtime. Obtain it from the [ICP dashboard](https://dashboard.internetcomputer.org/) (look up your canister, note the subnet it's hosted on) or query the NNS registry. Using `Principal::anonymous()` will produce receipts with an invalid subnet field that cannot be verified.
This computes the initial state hash and publishes the certified commitment.
### post_upgrade()
Call `mktd02::on_post_upgrade()` after schema migration but before any PII reads:
```rust
let adapter = MyAdapter;
let module_hash = [0u8; 32]; // ⚠ DEV ONLY — see Module Hash: Deployment Patterns for production
let config = MktdConfig {
    base_memory_id: 100,
    subnet_id: Principal::from_text("jtdsg-3h6gi-...").unwrap(),
};
MEMORY_MANAGER.with(|mm| {
    mktd02::on_post_upgrade(&adapter, &mm.borrow(), config, module_hash);
});
```
This detects manifest changes and triggers the recomputation cascade. Module hash is updated unconditionally on every `post_upgrade`.
> **Critical:** Any PII migration writes must happen BEFORE this call so the hash computation reflects the migrated state.
### After every PII write
Call `mktd02::refresh_state_hash()` after every successful PII-mutating write:
```rust
// In upsert_profile() or equivalent:
profile_cell.set(updated_profile)?;
mktd02::refresh_state_hash(&MyAdapter);
```
The host canister is responsible for calling `refresh_state_hash()`. The library does not auto-hook into writes.
