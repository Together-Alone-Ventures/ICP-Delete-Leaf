## Reference Adapters Provided

| Adapter | Description | Location |
|---|---|---|
| **StableCell adapter** | Reference impl for per-user canister with StableCell. Matches DaffyDefs pattern. | `examples/adapter_stable_cell.rs` |
| **StableBTreeMap adapter** | Stub for MKTd03 Tree mode (multi-record canisters). Deferred. | `examples/adapter_stable_btreemap.rs` |
| **Webhook export** | HTTP outcall example for JSON receipt export to external SIEM. | `examples/webhook_export.rs` |
| **DaffyDefs integration** | Complete working integration with ProfileAdapter, guard function, and lifecycle hooks. | `daffydefs/src/profile_canister/src/lib.rs` |

The DaffyDefs integration is the recommended starting point for new integrators. It demonstrates the complete pattern: adapter implementation, lifecycle hooks, guard function (Approach A), API endpoints, and receipt retrieval.
