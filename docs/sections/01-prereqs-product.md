### MKTd02-Specific Prerequisites

In addition to the platform prerequisites above, MKTd02 requires:

| Prerequisite | Detail | Responsibility |
|---|---|---|
| **Available MemoryId range** | 8 contiguous MemoryIds for MKTd02 (default: 100–107). Must not overlap with existing canister stable memory allocations. | Enterprise dev team |
| **Leaf mode architecture** | One data subject per canister. The canister holds PII for a single user/entity. Multi-subject canisters require MKTd03 (Tree mode) instead. | Enterprise architect |
| **Cycle cost per CVDR** | Cycle cost is workload-dependent and should be measured against your canister's actual deletion volume. No fixed estimate is published; measure on a local or test replica before sizing operational budgets. | Enterprise ops |
