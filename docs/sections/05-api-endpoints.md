## Expose MKTd02 Canister API

Add the following Candid endpoints to the canister. These are additive — the canister's existing API is unchanged.

| Endpoint | Type | Access | Purpose |
|---|---|---|---|
| `delete_profile()` (or `mktd_delete`) | Update | Restricted | Triggers full deletion flow. Returns `receipt_id` (hex string). |
| `mktd_get_state_hash()` | Query | Public | Returns state hash + optional ICP certificate (via `data_certificate()`). |
| `mktd_get_tombstone_status()` | Query | Public | Returns `{ is_tombstoned: bool, tombstoned_at: Option<u64> }`. |
| `mktd_get_receipt(id_hex)` | Query | Public | Returns full CVDR by `receipt_id`. All hashes as hex strings for readability. |
| `mktd_receipt_count()` | Query | Public | Number of stored receipts (0 or 1 for Leaf mode). |
