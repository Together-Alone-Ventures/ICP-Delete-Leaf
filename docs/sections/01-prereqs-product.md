### MKTd02-Specific Prerequisites

In addition to the platform prerequisites above, MKTd02 requires:

| Prerequisite | Detail | Responsibility |
|---|---|---|
| **Available MemoryId range** | 7 contiguous MemoryIds for MKTd02 (default: 100–106). Must not overlap with existing canister stable memory allocations. | Enterprise dev team |
| **Leaf mode architecture** | One data subject per canister. The canister holds PII for a single user/entity. Multi-subject canisters require MKTd03 (Tree mode) instead. | Enterprise architect |
