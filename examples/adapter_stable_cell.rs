//! # Reference Adapter: StableCell Pattern (DaffyDefs)
//!
//! Reference `MKTdDataSource` implementation for a canister using
//! `StableCell` with a single profile struct. This matches the
//! DaffyDefs profile_canister pattern.
//!
//! ## Canonicalisation Checklist
//!
//! - [x] All PII fields listed in `pii_field_manifest()` in field_order
//! - [x] `get_state_bytes()` uses `encode_pii_state()` (deterministic CBOR)
//! - [x] `tombstone_state()` writes TOMBSTONE_CONSTANT to every PII field
//! - [x] `is_tombstoned()` checks all PII fields == TOMBSTONE_CONSTANT
//! - [x] Field order in manifest matches field order in serialisation
//!
//! ## PII vs Non-PII Fields
//!
//! For DaffyDefs' StoredProfile:
//! - **PII (covered by MKTd02):** email, birthdate, gender, display_name
//! - **Non-PII (survives tombstone):** owner, state
//!
//! The `state` field is changed to `Deleted` during tombstoning, but it
//! is not a PII field — it's operational metadata.

// NOTE: This is a reference example, not compiled into the library.
// Adapt this pattern for your own canister.

use mktd02::trait_def::{CommitMode, MKTdDataSource};
use zombie_core::manifest::{compute_manifest_hash, FieldDescriptor};
use zombie_core::serialisation::encode_pii_state;
use zombie_core::tombstone::tombstone_constant;
use serde::{Deserialize, Serialize};

/// The PII-only subset of the profile, in canonical field_order.
/// This struct is what gets CBOR-encoded for state hashing.
#[derive(Serialize, Deserialize)]
struct PiiState {
    email: String,        // field_order: 0
    birthdate: String,    // field_order: 1
    gender: String,       // field_order: 2
    display_name: String, // field_order: 3
}

/// Example adapter wrapping a StableCell<StoredProfile>.
/// In practice, this reads from and writes to your thread_local! StableCell.
struct ProfileAdapter {
    // In real code, this accesses thread_local! PROFILE cell
}

impl MKTdDataSource for ProfileAdapter {
    fn mode(&self) -> CommitMode {
        CommitMode::Leaf
    }

    fn pii_field_manifest(&self) -> Vec<FieldDescriptor> {
        vec![
            FieldDescriptor {
                field_name: "email".into(),
                field_type: "String".into(),
                field_order: 0,
            },
            FieldDescriptor {
                field_name: "birthdate".into(),
                field_type: "String".into(),
                field_order: 1,
            },
            FieldDescriptor {
                field_name: "gender".into(),
                field_type: "String".into(),
                field_order: 2,
            },
            FieldDescriptor {
                field_name: "display_name".into(),
                field_type: "String".into(),
                field_order: 3,
            },
        ]
    }

    fn manifest_hash(&self) -> [u8; 32] {
        compute_manifest_hash(&self.pii_field_manifest())
    }

    fn get_state_bytes(&self) -> Vec<u8> {
        // Read from your StableCell, extract PII fields in canonical order
        let pii = PiiState {
            email: todo!("read from PROFILE cell"),
            birthdate: todo!("read from PROFILE cell"),
            gender: todo!("read from PROFILE cell"),
            display_name: todo!("read from PROFILE cell"),
        };
        encode_pii_state(&pii).expect("PII state encoding failed")
    }

    fn tombstone_state(&mut self) {
        let tc = tombstone_constant();
        let tc_str = hex::encode(tc);
        // Write tombstone constant to every PII field:
        // profile.email = tc_str.clone();
        // profile.birthdate = tc_str.clone();
        // profile.gender = tc_str.clone();
        // profile.display_name = tc_str.clone();
        // profile.state = ProfileState::Deleted;
        // PROFILE.with(|p| p.borrow_mut().set(profile));
        todo!("write tombstone constant to all PII fields, set state to Deleted")
    }

    fn is_tombstoned(&self) -> bool {
        let tc = tombstone_constant();
        let tc_str = hex::encode(tc);
        // Read profile from PROFILE cell, check all PII fields:
        // profile.email == tc_str
        //   && profile.birthdate == tc_str
        //   && profile.gender == tc_str
        //   && profile.display_name == tc_str
        todo!("check all PII fields == tombstone constant hex")
    }
}
