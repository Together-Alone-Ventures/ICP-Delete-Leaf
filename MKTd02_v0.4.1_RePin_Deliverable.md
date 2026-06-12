# MKTd02 Engine v0.4.1 Re-Pin — DELIVERABLE (inspect-and-propose)

**Packet:** `P0x_v0.4.1_RePin_BuildPacket` (high-assurance, patch-class conditional).
**Mode:** inspect-and-propose. No commit / push / tag — routes through the review ladder.
**Repo:** `ICP-Delete-Leaf` @ HEAD `209e72cc` (descends `ade76a9`, which carries the build-guide
permissions fix; engine released tag `mktd02-v0.4.0` = `ba5cef5`).

**Prime Directive (G 1d):** dependency/API-compatibility work only. Preserve the public API, receipt
schema, every hash formula, and the 8-slot stable-memory layout 100–107 *including the serialized
encoding of each stable structure*. Any forced change to those ⇒ **HALT and report**.

---

## STATUS: PHASE 1 COMPLETE — PAUSED FOR APPROVAL

No manifest has been modified. Phase 2 (W2 bump / W3 adapt / W4 lock / W5 build) begins only after this
dependency-chain report is approved.

---

## PHASE 1 (W1) — Dependency-graph inspection

### Crates that pull `ic-cdk` / `ic-stable-structures`

| Crate | Workspace / lockfile | `ic-cdk` | `ic-stable-structures` | Needs bump? |
|---|---|---|---|---|
| `mktd02` (lib) | **root** workspace → root `Cargo.lock` | `0.17` (locked 0.17.2) | `0.6` (locked 0.6.9) | **YES** |
| `mktd02-macros` (proc-macro) | root workspace | — | — | no (syn/quote/proc-macro2 only) |
| `mktd02-harness` | **separate** workspace → `harness/Cargo.lock` | `0.17` | `0.6` | **YES** (independent lock) |
| `zombie-core` v0.3.1 | external git dep (tag) | **— none —** | **— none —** | **NO** |

Evidence:
- `mktd02-macros/Cargo.toml` deps = `syn`, `quote`, `proc-macro2`; `grep ic_cdk|ic_stable` in
  `mktd02-macros/src` → none.
- `harness/Cargo.toml` is a **self-contained workspace** (empty `[workspace]` table) with its OWN
  `harness/Cargo.lock`, deliberately so it "does NOT perturb the root workspace's Cargo.lock." It pins
  `ic-cdk = 0.17`, `ic-stable-structures = 0.6`, `pocket-ic = 11.0.0`, `mktd02` (path), `zombie-core`
  (tag v0.3.1). Used by `harness/src/lib.rs` + `harness/tests/runtime.rs`.
- `zombie-core` v0.3.1 (`508f2f8`): `Cargo.toml` deps = `sha2`, `ciborium`, `serde`, `candid 0.10`,
  `hex` — **no `ic-cdk`, no `ic-stable-structures`**. `grep -rE 'ic-cdk|ic_stable_structures|Storable|
  memory_manager|StableCell|StableBTreeMap'` over `zombie-core/src` → **0 matches**. It implements no
  `Storable` and touches no stable memory (its receipt types are CBOR via `ciborium`). Self-described
  "pure Rust with zero ICP dependencies" — confirmed.

### Does zombie-core need its own re-pin + tag first?

**No.** zombie-core carries neither dependency, so the is-0.6→0.7 / ic-cdk 0.17→0.18 bump does not reach
it. The chain `zombie-core → new tag → engine re-points` is **not required** by this packet. zombie-core
stays frozen at **`zombie-core-v0.3.1`** — which also keeps host/engine zombie-core type-unification
trivial (the host pins the same tag).

> Note (observation, out of scope): zombie-core's `Cargo.toml` `package.version = "0.3.0"` while its tag
> is `zombie-core-v0.3.1` (root `Cargo.lock` records `version = "0.3.0"` at the v0.3.1 git rev). Pre-
> existing cosmetic mismatch, unrelated to this re-pin. Not touched.

### Dependency chain (engine side only)

```
root workspace ─┬─ mktd02      ── ic-cdk 0.17  ── ic-cdk-executor 0.1.0   ┐ links="ic-cdk async executor"
                │              ── ic-stable-structures 0.6                 │  (collides with host's 0.18)
                │              ── zombie-core v0.3.1 (no ICP deps)         │
                └─ mktd02-macros (no ICP deps)                            │
harness workspace ─ mktd02-harness ── ic-cdk 0.17 / is 0.6 / pocket-ic 11 ┘  (own lock)
```

After the bump: `ic-cdk 0.18 → ic-cdk-executor 1.x` and `is 0.7`. In the host graph OpenChat already
uses ic-cdk 0.18 → ic-cdk-executor 1.0.2, so engine + host then resolve to a **single**
`ic-cdk-executor` (the `links` collision disappears) and a single `MemoryManager<DefaultMemoryImpl>`
type (is-0.7 on both sides). This is exactly what unblocks Divergence #1.

### Proposed sequencing

1. **No zombie-core step.** Leave `zombie-core-v0.3.1`.
2. **Single coordinated engine bump** of two manifests in lockstep — `mktd02/Cargo.toml` (root
   workspace) and `harness/Cargo.toml` (separate workspace). They share the engine source (`harness`
   depends on `mktd02` by path), so they must adapt together.
3. W2 manifests → W3 compile-adapt → W4 lockfiles (root `Cargo.lock` *and* `harness/Cargo.lock`, each
   its own deliberate commit; fold the parked stale-vs-manifest fix into the root one) → W5 build +
   lib 9/9 + PocketIC 3/3 → E4-lite.

### Phase-2 risk surface (flagged now, NOT acted on)

- **is 0.6→0.7 `Storable`/`Bound` (Prime-Directive critical):** all 6 `Storable` impls live in
  `mktd02/src/storage.rs` — `Hash32`, `MetaCell`, `StorableU64`, `StorableBool`, `OptionalTimestamp`,
  `ReceiptBytes`. The packet's named-critical ones map to slots: `StorableBool` → base+5
  `finalization_lock`; `Hash32` (key) + `ReceiptBytes` (value) → base+6 receipt store;
  `OptionalTimestamp` → base+7 `tombstoned_at`. W3 must keep every `to_bytes`/`from_bytes` byte-
  identical; if is-0.7's trait shape forces an emitted-bytes change ⇒ **HALT (1d)**.
- **`MemoryManager`/`StableCell`/`StableBTreeMap` API:** `storage.rs` `setup_storage` (slot wiring
  100–107) and `lib.rs` init/post_upgrade signatures. Adapt call sites only; slot map must be
  identical.
- **ic-cdk 0.17→0.18 internal calls** (no public-API surface): `set_certified_data`/`data_certificate`
  (certified.rs), `caller`/`is_controller` (finalization.rs), `id`/`api::time`/`trap`/instruction
  counters (engine.rs, lib.rs, storage.rs traps). Several were renamed in 0.18 (e.g.
  `set_certified_data` → `certified_data_set`, `ic_cdk::caller()` → `api::msg_caller()`,
  `ic_cdk::id()` → `api::canister_self()`). These are **internal** to the library and do not appear in
  any Candid signature.
- **Candid-diff clarification needed:** `mktd02` is a **library** crate — it exposes **no `#[update]`/
  `#[query]` endpoints and generates no `.did`** of its own (the host canister wraps the public Rust
  fns). The "Candid interface unchanged" check therefore = the **public Rust API signatures**
  (`init`, `on_post_upgrade`, `execute_deletion[_with_record_id]`, `get_pending_certificate`,
  `finalize_receipt[_after_host_authorization]`, queries) must be unchanged. If the packet's
  "generated `.did`" refers to the harness cdylib, I'll diff that; otherwise I'll attest the public Rust
  surface. **Confirm intent before W3 sign-off.**

---

## PHASE 2 — W2 / W3 / W4 / W5  — COMPLETE (inspect-and-propose; uncommitted)

**STATUS:** all changes applied to the working tree (no commits). Build green; lib **9/9**, PocketIC
**3/3**, wasm32 clean. The 1d Prime Directive holds: **no API / receipt-schema / hash-formula /
stable-memory-layout change** — evidence below.

### W2 — Manifest bump (2 files, no other dep changes)

`mktd02/Cargo.toml` and `harness/Cargo.toml`: `ic-cdk "0.17" → "0.18"`, `ic-stable-structures
"0.6" → "0.7"`. Nothing else changed. zombie-core stays `zombie-core-v0.3.1`. No *forced* extra
manifest dep changes. Resolved graph: ic-cdk 0.18.7, ic-cdk-executor 1.0.2, ic-stable-structures
0.7.2, ic0 1.1.0 (+ ic-cdk's own new transitives: thiserror/strum/darling/ic-error-types/…).
**candid stays 0.10.23, ciborium 0.2.2, serde 1.0.228, sha2 0.10, zombie-core @508f2f8b — all
identical** (the entire receipt / hash / serialization path is on these unchanged crates).

### W3 — Per-change 1d-attestation table

| # | File(s) | Change | Forced by | Evidence it does NOT touch API / schema / hash / layout |
|---|---|---|---|---|
| 1 | `storage.rs` ×6 `Storable` impls | add `fn into_bytes(self) -> Vec<u8> { self.to_bytes().into_owned() }` | is-0.7 added `into_bytes` as a **required** trait method (no default) | `into_bytes` **delegates to the unchanged `to_bytes`** → the bytes written to stable memory are bit-for-bit what v0.4.0 wrote. `to_bytes`/`from_bytes` bodies untouched (diff shows no edit to them). `const BOUND` untouched (table below). Lib tests `meta_cell_decodes_legacy_49_byte_layout` + `meta_cell_roundtrip_preserves_pending_receipt_id` pass under is-0.7 — direct byte-layout lock. |
| 2 | `storage.rs` `setup_storage` | drop `.expect(...)` on 7 `StableCell::init` | is-0.7 `Cell::init` returns `Self` (was `Result`) | Pure call-site shape; slot ids `base..=base+7` (= 100..=107) literally unchanged; no bytes involved. Collision/schema gates below it unchanged. |
| 3 | `engine.rs`(6) `storage.rs`(4) `state.rs`(2) `certified.rs`(1) `nonce.rs`(1) — 14 sites | `.set(X).expect(msg)` → `.set(X)` | is-0.7 `Cell::set` returns old `T` (was `Result`), not `#[must_use]` | Write side-effect identical (`set` still flushes `value.to_bytes()` to the same slot); only the discarded return type changed. On valid (in-bounds) data behaviour is identical; the sole difference is the panic *message* on an impossible out-of-bounds write. |
| 4 | `certified.rs:51` | `ic_cdk::api::set_certified_data(&c)` → `certified_data_set(c)` | ic-cdk 0.18 deprecation | **Both bodies call `ic0::certified_data_set(buf)` with the same 32 bytes** (verified in ic-cdk-0.18.7 `api.rs`). V2 / Phase-B certified-data path behaviour-identical. `data_certificate()` unchanged. |
| 5 | `engine.rs:57`, `finalization.rs:179` | `ic_cdk::caller()` → `api::msg_caller()` | ic-cdk 0.18 deprecation | Both copy `ic0::msg_caller`. `engine.rs`: `record_id` from `caller().as_slice()` → identical bytes. `finalization.rs`: controller guard + `NoPendingReceipt`-before-`NotController` ordering unchanged. |
| 6 | `engine.rs:88`, `state.rs:16` | `ic_cdk::id()` → `api::canister_self()` | ic-cdk 0.18 deprecation | Both copy `ic0::canister_self`. `state.rs`: the salt preimage `hash_with_tag(TAG_SALT, &[<principal bytes>])` is **byte-identical** → state-hash formula unchanged. `engine.rs`: receipt `canister_id` identical. |

> Notes: `ic_cdk::trap`, `ic_cdk::api::time`, `ic_cdk::api::is_controller`, `ic_cdk::api::data_certificate`
> are **not** deprecated in 0.18 and are unchanged. No hashing/preimage code (`hashing.rs`/`certified.rs`
> `compute_*`/`engine.rs` tombstone & event-hash preimages) was edited — confirmed by `git diff`.

### Stable-structure encoding + Bound attestation (per impl)

`Bound` values are **literally unchanged** (the `const BOUND` lines have zero diff). `to_bytes`/`from_bytes`
unchanged; `into_bytes` delegates to `to_bytes`.

| `Storable` impl | Slot(s) | `max_size` | `is_fixed_size` | Encoding evidence |
|---|---|---|---|---|
| `StorableBool` | **base+5 `finalization_lock`** | 1 | true | 1 byte `0/1`; `into_bytes`=`to_bytes`. |
| `Hash32` (key) + `ReceiptBytes` (val) | **base+6 receipt store** | 32 / 8192 | true / false | key 32B fixed, value CBOR ≤8192; both `into_bytes`=`to_bytes`. **BTreeMap node layout = f(BOUND, page-size) — BOUND unchanged + format constants identical (below).** |
| `OptionalTimestamp` | **base+7 `tombstoned_at`** | 9 | true | 1B tag + 8B LE; `into_bytes`=`to_bytes`. |
| `MetaCell` | base+0 | 82 | false | 49/82-byte LE layout; locked by passing round-trip tests. |
| `Hash32` | base+1/+3/+4 | 32 | true | raw 32B. |
| `StorableU64` | base+2 | 8 | true | 8B LE. |

**On-disk container format unchanged (is-0.6.9 vs is-0.7.2 source):**
- `Cell`: `MAGIC = b"SCL"`, `HEADER_V1_SIZE = 8`, `LAYOUT_VERSION = 1`, value written at offset 8 — **identical**.
- `StableBTreeMap`: `MAGIC = b"BTR"`, `LAYOUT_VERSION = 1`/`LAYOUT_VERSION_2 = 2`, identical `Version::V1/V2`
  page-size derivation and `Node::new_v1/new_v2` — **identical**.
- ⇒ a canister holding v0.4.0 (is-0.6) stable data is read correctly by v0.4.1 (is-0.7): **upgrade-safe**,
  no migration, no re-encode. Corroborated end-to-end by lib 9/9 + PocketIC 3/3 (cells + receipt store
  written and read back under is-0.7).

### API-surface attestation (1d "public interface unchanged")

`cargo public-api` is not installed in this environment, so: **`git diff` shows `mktd02/src/lib.rs`
(the public-API surface) and `mktd02/src/trait_def.rs` (the `MKTdDataSource` trait) are byte-identical
(zero diff).** A grep for changed `pub fn|struct|enum|trait|const|type` across `mktd02/src/` returns
**none**. Full public surface (unchanged), enumerated:

```
pub mod certified|engine|export|finalization|guard|nonce|state|storage|trait_def
pub use engine::DeletionError
pub use finalization::{FinalizationError, PendingCertificate}
pub use trait_def::{CommitMode, GuardError, MKTdDataSource}
pub use zombie_core::{DeletionReceipt, FieldDescriptor, ProtocolVersion, ReceiptSummary}
pub struct MktdConfig { base_memory_id: u8 }
pub fn init / on_post_upgrade / execute_deletion / execute_deletion_with_record_id
pub fn get_pending_certificate / finalize_receipt / finalize_receipt_after_host_authorization
pub fn is_pending_finalization / is_tombstoned / is_initialised / get_state_hash
pub fn get_certified_state_hash / get_receipt / get_receipt_summary / get_tombstone_status
pub fn refresh_state_hash / receipt_count / assert_can_write
```
A1/A2 entrypoints, the shared `finalize_locked_receipt` helper visibility (private, unchanged), and the
manifest/`FieldDescriptor` types (re-exported from the unchanged zombie-core) are all intact.

**Candid cross-check (corroborating, not the contract):** `mktd02` is a library — no `.did`. The harness
cdylib (`harness/src/lib.rs`) exposes `#[ic_cdk::*]` methods, and **its source is unchanged** (zero diff),
so its Candid surface is unchanged by construction; it builds to wasm32 clean and its 3 PocketIC cases
(which Candid-encode/decode every call) pass. candid itself is **0.10.23 → 0.10.23** (no version move).

### Serialization / receipt-schema attestation

The `DeletionReceipt` struct, its CBOR (ciborium) and canonical-JSON (serde) encodings, and every hash
formula live entirely in **zombie-core @ `508f2f8b` (unchanged)** + **ciborium 0.2.2 / serde 1.0.228 /
candid 0.10.23 (all unchanged in the lock, HEAD vs now)**. No field order, type, or encoder moved.
⇒ receipt wire/disk bytes and the canonical JSON export are **byte-identical**.

### W4 — Cargo.lock (two lockfiles, each its own commit)

- **root `Cargo.lock`** and **`harness/Cargo.lock`** regenerated for the new graph. Each is a **separate
  deliberate commit** (per packet), ordered after the W2/W3 source commit.
- Both pass `cargo build --locked` (root + harness) → the locks match their manifests exactly.
- The lock delta is **only** the ic-cdk/is dependency tree (ic-cdk 0.18.7, ic-cdk-executor 1.0.2,
  ic-stable-structures 0.7.2, ic0 1.1.0, + ic-cdk's new transitives) — candid/ciborium/serde/sha2/
  zombie-core entries are untouched.
- **Stale-vs-manifest fix:** the regenerated, `--locked`-clean root lock is by definition consistent
  with the manifest, subsuming the parked drift. (Reviewer: confirm the specific parked entry — if it
  was tracked elsewhere — is absent from the regenerated lock.)

### W5 — Build + tests (evidence)

| Check | Result |
|---|---|
| `cargo build` (root workspace) | clean — `mktd02` compiles under ic-cdk 0.18.7 / is-0.7.2 |
| `cargo test -p mktd02` | **9 passed; 0 failed** (engine lib) |
| harness `cargo build` (host) | clean |
| harness `cargo build --target wasm32-unknown-unknown --release` | clean (`mktd02_harness.wasm`, 1.0 MB) |
| `POCKET_IC_BIN=… cargo test` (harness) | **3 passed; 0 failed** (`a1_wrapper_parity…`, `a2_host_supplied_record_id…`, `default_path_controller_guard_two_cells`) |
| `cargo build --locked` (root + harness) | clean (lock ↔ manifest consistent) |

### Exit gate — E4-lite (G 1c) — re-confirmation

| Property | Status under v0.4.1 |
|---|---|
| receipt-id binding (`record_id` in v3 preimage) | preserved — `receipt_id_is_sensitive_to_record_id` ✓; `a2_host_supplied_record_id…` ✓ |
| finalization lock / single-shot | preserved — `host_auth_finalize_succeeds_and_releases_lock` ✓ |
| no-lock / mismatch / double-finalize | preserved — `host_auth_no_lock…`, `…mismatch…`, `…double_finalize…` ✓ |
| controller path + guard ordering | preserved — `default_path_controller_guard_two_cells` ✓ (NoPendingReceipt-before-NotController unchanged) |
| A2 host-authorized path | preserved — A2 tests ✓ |
| stable-memory slot map 100–107 (+5/+6/+7) | preserved — format + BOUND identical; slot ids unchanged |
| DaffyDefs compat (sets up) | engine is is-0.7/ic-cdk-0.18 — unblocks R2 (DaffyDefs move, separate) |

No forced protocol/schema/hash/API/layout change surfaced ⇒ **1d not tripped**; full adversarial model
not reopened (per 1c).

### Acceptance-condition mapping

1. builds clean on ic-cdk 0.18 / is-0.7 — ✅ (zombie-core untouched). 2. 1d attested w/ evidence —
✅ (tables above; Candid surface unchanged; encodings byte-identical; slots 100–107 unchanged).
3. lib 9/9 + PocketIC 3/3 — ✅. 4. E4-lite — ✅. 5. Cargo.lock deliberate (×2) + `--locked` clean —
✅. **6/7 (tag + Arweave + guide; DaffyDefs move)** — out of this packet's edit scope (R1/R2);
proceed via the release ladder after Claude+CD review.

### Out-of-scope / HALT checks — none tripped

No protocol/schema/hash/API/layout change was forced. Module-hash V3 wiring (P1A), DaffyDefs migration
(R2), and OpenChatZD P1 (R3) remain separate. No commit/push/tag performed.

---

## Post-review amendments (CD independent review — folded per G E4-lite condition 1)

1. **BTreeMap wording corrected.** ic-stable-structures 0.6.9 and 0.7.2 are *not*
   source-identical overall. The correct claim, verified by CD against the 0.7.2 sources, is:
   the **on-disk format and receipt-store derivation are compatible/byte-identical for this
   bound shape** (key 32 fixed; value 8192 bounded non-fixed) — BTR magic, layout versions 1/2,
   header size/offsets, V2 page-size derivation, and node read/write layout.
2. **candid lockfile distinction recorded.** Root `Cargo.lock` carries candid **0.10.23**;
   `harness/Cargo.lock` carries candid **0.10.29**. Both pre-date and are unchanged by this
   re-pin. Statements above reading "candid 0.10.23 unchanged" refer to the root lock.
