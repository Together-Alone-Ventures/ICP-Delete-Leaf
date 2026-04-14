## Residual Trust Statement

A MKTd02 CVDR is a cryptographic proof of a state transition within a defined system boundary. It proves that, at a recorded point in time, under a specific version of code attested by ICP subnet BLS signatures, the canister's PII fields transitioned from an active state to the tombstone constant, and that state was committed to ICP's certified state tree.

It does not prove that no copy of the data exists anywhere outside the canister, that physical storage bytes were scrubbed, or that snapshot backups were destroyed.

A CVDR is independently verifiable without requiring operational participation from Together Alone Ventures or the data controller, subject to the residual trust assumptions documented below.

The following table documents the assumptions a verifying party must accept when relying on a MKTd02 CVDR, together with their mitigations and severity.

| # | Assumption | Mitigation | Severity |
|---|---|---|---|
| **RT1** | ICP subnet consensus is honest (≥ 2/3 of nodes not colluding). | Foundation of ICP's security model. Subnet membership is governed by the NNS. For sovereign subnets or UTOPIA deployments where the operator controls all nodes, consensus becomes self-attestation — the independent receipt structure retains evidentiary value but the BLS trust anchor is weakened. | Foundational |
| **RT2** | The canister controller does not upgrade the canister to reverse tombstones after deletion. | MKTd02 provides **detectable resurrection, not prevention**. The module hash recorded in the deletion receipt reflects the code version at deletion time. Any later code change is attributable when live module-hash corroboration, build provenance records, or subsequent receipts are examined — it is not automatically surfaced by the original receipt alone. The original receipt remains valid evidence of deletion at time T under code version M. DAO governance or an immutable controller mitigates the risk of unauthorised upgrades. | Medium |
| **RT3** | The `MKTdDataSource` adapter faithfully maps all PII fields to the state bytes used for hashing and tombstoning. | The adapter is open-source and auditable. It runs inside the canister under ICP consensus. An incorrect adapter that omits a PII field produces a state hash that does not cover that field — detectable by any party who independently checks a record against the published State Encoding Spec. As noted in the verifier scope boundary: verifier tooling validates cryptographic consistency of receipts; completeness of PII field mapping requires source code audit of the adapter. | Medium |
| **RT4** | Tombstone writes effect logical removal of PII from the governed data structure. | MKTd02 operates at the logical data-structure level — it proves the data structure no longer contains the value and that this is attested by ICP subnet consensus. Whether the underlying storage medium retains prior byte patterns in freed or overwritten regions is a platform-level concern outside MKTd scope. Physical byte scrubbing, where required by applicable guidance, should be addressed as a separate operational control. | Low |
| **RT5** | No canister snapshot or backup contains a copy of the pre-deletion state that could be restored. | ICP supports canister snapshots for backup and migration. A snapshot taken before deletion and restored after would replace the tombstoned state. Snapshot restoration changes the certified commitment, which is detectable by verifiers comparing the receipt's post-state against the canister's current state. Snapshot management and retention policies are an operational obligation of the data controller, outside MKTd scope. | High |

### Scope of verification tooling

CVDR-Verify validates the cryptographic and consistency properties of a receipt (V1–V4). It does not verify completeness of PII field mapping (RT3), physical byte scrubbing (RT4), or snapshot management (RT5). Those require operational controls and source code audit of the adapter.

### What this means for a data controller

A data controller relying on MKTd02 CVDRs for GDPR Article 17 compliance should:

- Maintain open-source adapter code to support RT3 auditability.
- Implement DAO or multi-sig controller governance and document upgrade authority to address RT2.
- Establish, document, and enforce snapshot retention and deletion policies to address RT5.
- Treat physical storage scrubbing (RT4) as a separate operational control where required by applicable guidance.
