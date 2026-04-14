**Sync note:** The residual trust table and mitigations in this document are kept in sync with `docs/sections/12-residual-trust.md`, which is the Integration Guide source for the same material. If you update one, update the other.

# MKTd02 Residual Trust Statement

**Protocol:** MKTd02 v0.3.x (Leaf mode)  
**Issued by:** Together Alone Ventures OÜ  
**Audience:** Data Protection Officers, legal counsel, compliance auditors, data protection regulators  
**Repository:** This document is published in the MKTd02 repository and is the authoritative trust boundary statement for this protocol version. For technical integration details, see `MKTd02_Integration_Guide.md` in the repository root and the source documentation under `docs/sections/`.

---

## Purpose

This document defines precisely what a MKTd02 Cryptographically Verifiable Deletion Receipt (CVDR) proves, what it does not prove, and what residual trust assumptions a relying party must independently manage.

It is written for a non-technical audience and does not assume knowledge of the Internet Computer Protocol (ICP) or cryptography.

---

## What a MKTd02 CVDR Proves

A MKTd02 CVDR is a cryptographic proof of a state transition within a defined system boundary. A CVDR is independently verifiable — any party with the receipt and the CVDR-Verify tooling can confirm its validity without contacting Together Alone Ventures or the data controller. This independence is subject to the residual trust assumptions documented below.

Specifically, a finalized CVDR proves the following:

**1. Deletion occurred at a recorded time.**  
At the timestamp in the receipt, the canister's PII fields transitioned from an active state to a permanent tombstone marker.

**2. The transition was attested by decentralised consensus.**  
The Internet Computer subnet's BLS threshold signature — requiring agreement from at least two-thirds of the subnet's independent nodes — attests to the post-deletion state. This is not a claim made solely by the data controller.

**3. The post-deletion state is cryptographically bound to the receipt.**  
Any subsequent change to the canister's certified state is detectable by comparing the receipt's recorded post-state against the canister's current state.

**4. The code version at deletion is recorded and auditable.**  
The module hash in the receipt identifies the exact version of code running at deletion time. This code version is intended to be traceable through published source, build provenance, and release artifacts maintained by the project. Any later code change produces a different module hash, making the change attributable when live module-hash corroboration or build provenance records are examined.

---

## What a MKTd02 CVDR Does Not Prove

A CVDR does not prove the following:

- That no copy of the data exists outside the canister (e.g. in backups, logs, downstream systems, or third-party processors).
- That the physical storage medium's bytes were overwritten or scrubbed (see RT4).
- That canister snapshots or backups taken before deletion were also destroyed (see RT5).
- That all PII fields were correctly identified and included in the deletion scope — this depends on the correctness of the integration adapter code (see RT3).
- That resurrection is impossible — the data controller retains the standard ICP ability to upgrade canister code. Resurrection is detectable, not cryptographically prevented (see RT2).

*The appropriate framing for regulatory purposes is: a MKTd02 CVDR provides strong, independently verifiable evidence that a deletion operation was executed at a specific time, under a specific version of auditable code, with the resulting state attested by decentralised ICP subnet consensus. It is one component of a broader data deletion compliance programme.*

---

## Residual Trust Assumptions

The following table documents the assumptions a verifying party must accept when relying on a MKTd02 CVDR.

| # | Assumption | Mitigation | Severity |
|---|---|---|---|
| **RT1** | ICP subnet consensus is honest (≥ 2/3 of nodes not colluding). | Foundation of ICP's security model. Subnet membership is governed by the NNS on-chain governance system. For sovereign or UTOPIA deployments where the operator controls all nodes, consensus becomes self-attestation — the independent receipt structure retains evidentiary value but the BLS trust anchor is weakened. | Foundational |
| **RT2** | The canister controller does not upgrade the canister to reverse tombstones after deletion. | MKTd02 provides detectable resurrection, not prevention. The module hash recorded in the deletion receipt reflects the code version at deletion time. Any later code change is attributable when live module-hash corroboration, build provenance records, or subsequent receipts are examined — it is not automatically surfaced by the original receipt alone. The original receipt remains valid evidence of deletion at time T under code version M. DAO governance or an immutable controller mitigates the risk of unauthorised upgrades. | Medium |
| **RT3** | The adapter (integration code) faithfully maps all PII fields to the state bytes used for hashing and tombstoning. | The adapter is open-source and auditable. It runs inside the canister under ICP consensus. An incorrect adapter that omits a PII field produces a state hash that does not cover that field — detectable by any party who independently checks a record against the published state-encoding documentation. Completeness of PII field mapping requires source code audit of the adapter. | Medium |
| **RT4** | Tombstone writes effect logical removal of PII from the governed data structure. | MKTd02 operates at the logical data-structure level — it proves the data structure no longer contains the value and that this is attested by ICP subnet consensus. Whether the underlying storage medium retains prior byte patterns in freed or overwritten regions is a platform-level concern outside MKTd scope. Physical byte scrubbing, where required by applicable guidance, should be addressed as a separate operational control. | Low |
| **RT5** | No canister snapshot or backup contains a copy of the pre-deletion state that could be restored. | ICP supports canister snapshots for backup and migration. A snapshot taken before deletion and restored after would replace the tombstoned state. Snapshot restoration changes the certified commitment, which is detectable by verifiers comparing the receipt's post-state against the canister's current state. Snapshot management and retention policies are an operational obligation of the data controller, outside MKTd scope. | High |

---

## Verification Summary

Independent verification of a MKTd02 CVDR is performed using [CVDR-Verify](https://github.com/Together-Alone-Ventures/CVDR-Verify). The four verification checks address different aspects of the trust chain.

| Check | Name | What it verifies |
|---|---|---|
| **V1** | Hash consistency | Recomputes all receipt hash fields using published formulas. Confirms internal consistency. Does not require ICP connectivity. |
| **V2** | BLS certificate | Verifies the ICP subnet's BLS threshold signature and confirms the certified post-deletion state matches the receipt. Requires the embedded certificate (finalized receipts) or a live canister query (pending receipts). |
| **V3** | Module hash provenance | Confirms the module hash in the receipt can be traced to published build artifacts and open-source code. Supports code auditability of the deletion logic and the adapter. |
| **V4** | Tombstone persistence | Queries the live canister to confirm the tombstone is still present. A point-in-time check — not available after the canister is deleted, but V1–V3 remain valid indefinitely. |

---

## Guidance for Data Controllers

A data controller relying on MKTd02 CVDRs as part of a GDPR Article 17 compliance programme should address the following operational controls to complement the cryptographic assurances the receipt provides:

- **Adapter auditability (RT3):** Adapter code should be open-source and subject to periodic third-party audit.
- **Upgrade governance (RT2):** Canister upgrade authority should be governed by DAO governance, multi-signature control, or equivalent. The governance arrangement should be documented and auditable.
- **Snapshot policy (RT5):** Canister snapshot and backup retention policies should be documented, time-limited, and enforced. Snapshots containing pre-deletion state should be identified and destroyed as part of the erasure workflow.
- **Physical scrubbing (RT4):** Where applicable regulatory guidance requires physical storage scrubbing beyond logical deletion, this should be addressed at the infrastructure or platform level as a separate control.
- **Scope documentation:** The scope of the CVDR — which canister, which principal, which fields, which protocol version — is recorded in the receipt itself and in the adapter's published state-encoding documentation and associated release/build provenance for the adapter version in use.

---

## Disclaimer

This document is produced by Together Alone Ventures OÜ as a reference statement describing the trust properties of the MKTd02 protocol. It does not constitute legal advice and should not be relied upon as such. The suitability of MKTd02 CVDRs for any particular regulatory purpose is a matter for the data controller and their legal counsel to determine. Together Alone Ventures makes no warranty as to fitness for a particular purpose.

For technical integration guidance, see `MKTd02_Integration_Guide.md` in the repository root.
