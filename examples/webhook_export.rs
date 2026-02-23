//! # Receipt Export Webhook Example
//!
//! Complete example of pushing JSON receipts to an external endpoint
//! via ICP HTTP outcall, including error handling and retry.
//!
//! Requires the `json` feature flag on the `mktd02` crate:
//!
//! ```toml
//! [dependencies]
//! mktd02 = { path = "../MKTd02/mktd02", features = ["json"] }
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use mktd02::export;
//!
//! // After executing a deletion:
//! let receipt = mktd02::get_receipt(&receipt_id).unwrap();
//!
//! // Export as CBOR (always available)
//! let cbor_bytes = export::to_cbor_bytes(&receipt);
//!
//! // Export as JSON (requires "json" feature)
//! let json_str = export::to_json(&receipt);
//!
//! // Push to webhook (template -- customise for your SIEM)
//! // export::webhook_push(&receipt, "https://your-siem.example.com/api/ingest");
//! ```
//!
//! ## SIEM Integration Notes
//!
//! - **Splunk:** Use HEC (HTTP Event Collector) endpoint.
//!   Set Content-Type: application/json. Include index and sourcetype.
//! - **ELK/Elastic:** POST to _bulk or _doc endpoint.
//! - **Datadog:** Use Log Management API endpoint.
//!
//! ## ICP HTTP Outcall Considerations
//!
//! - HTTP outcalls require cycles payment (~0.4B cycles per call)
//! - Response body is limited to 2MB
//! - Subnet egress may have rate limits
//! - Implement retry with exponential backoff
//! - Consider batching receipts if processing multiple deletions
