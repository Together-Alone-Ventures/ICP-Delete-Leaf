## Receipt Export

A CVDR is defined by its fields and verification rules, not by a particular wire encoding. The canister stores and returns receipts as a Candid struct via `mktd_get_receipt()`. The export module provides alternate serialisations for interoperability.

| Function | Purpose |
|---|---|
| `mktd02::export::to_cbor_bytes(receipt)` | Returns the deterministic CBOR encoding of the receipt (the authoritative format). |
| `mktd02::export::to_json(receipt)` [feature: `json`] | Serialises to JSON for SIEM/GRC ingestion (Splunk, ELK, Datadog). |
| `mktd02::export::webhook_push(receipt, url)` [feature: `json`] | Template for HTTP outcall to external webhook. Handles ICP-specific concerns. |

The JSON export option is behind the `json` feature flag. Default builds exclude it to minimise WASM size.

**Receipt delivery:** MKTd02 generates and stores the receipt; how it reaches the data subject is the enterprise's UX decision. Options include: in-app display immediately after deletion, downloadable JSON file, email delivery, or integration with existing GDPR request management workflows. The receipt persists in the canister's stable memory indefinitely and remains queryable via `mktd_get_receipt()` even after tombstoning — the canister stays alive, only PII is erased.
