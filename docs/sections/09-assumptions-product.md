## Assumptions

### MKTd02-Specific

| # | Assumption | Implication |
|---|---|---|
| **A1** | Deletion is triggered by an authorised call. | No CDC, no polling, no external trigger. The caller initiates the request. |
| **A2** | State hash is continuously maintained. | `refresh_state_hash()` after every PII write. No separate pre-delete capture step. |
| **A3** | Canister holds data for a single data subject (Leaf mode). | Multi-subject canisters need MKTd03 (Tree mode). |
