//! PocketIC runtime harness for the three P0 cases that cannot run as ordinary
//! host unit tests (they depend on `ic_cdk::id/time/caller/is_controller`).
//!
//! Two-step run:
//!   1. cargo build --manifest-path harness/Cargo.toml --target wasm32-unknown-unknown --release
//!   2. POCKET_IC_BIN=<server> cargo test --manifest-path harness/Cargo.toml

use candid::{Decode, Encode, Principal};
use mktd02_harness::ReceiptFieldsDto;
use pocket_ic::PocketIc;

fn harness_wasm() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/target/wasm32-unknown-unknown/release/mktd02_harness.wasm"
    );
    std::fs::read(path).unwrap_or_else(|e| {
        panic!(
            "harness wasm not found at {path} ({e}). Build it first:\n  \
             cargo build --manifest-path harness/Cargo.toml --target wasm32-unknown-unknown --release"
        )
    })
}

/// Create + install a fresh harness canister. Controller is the anonymous
/// principal (PocketIC `create_canister` default), so any non-anonymous caller
/// is a clean non-controller.
fn install(pic: &PocketIc) -> Principal {
    let cid = pic.create_canister();
    pic.add_cycles(cid, 2_000_000_000_000);
    pic.install_canister(cid, harness_wasm(), Encode!().unwrap(), None);
    cid
}

// --- Case 1: A1 wrapper parity -------------------------------------------------
#[test]
fn a1_wrapper_parity_record_id_equals_caller_principal_bytes() {
    let pic = PocketIc::new();
    let cid = install(&pic);
    let caller = Principal::from_slice(&[0xA1, 0xA1, 0xA1, 0xA1]);

    let reply = pic
        .update_call(cid, caller, "h_execute_deletion", Encode!().unwrap())
        .expect("update_call failed");
    let receipt_id = Decode!(&reply, Result<Vec<u8>, String>)
        .unwrap()
        .expect("execute_deletion returned Err");

    let q = pic
        .query_call(
            cid,
            Principal::anonymous(),
            "h_get_receipt_fields",
            Encode!(&receipt_id).unwrap(),
        )
        .expect("query_call failed");
    let dto = Decode!(&q, Option<ReceiptFieldsDto>)
        .unwrap()
        .expect("receipt missing");

    // record_id == caller principal bytes (documented caller-derived behaviour)
    assert_eq!(dto.record_id, caller.as_slice().to_vec());
    assert_eq!(dto.canister_id, cid);
    // receipt identity consistent with the LIVE receipt-id formula
    let expected =
        zombie_core::receipt::compute_receipt_id(&dto.canister_id, &dto.record_id, dto.deletion_seq);
    assert_eq!(dto.receipt_id, expected.to_vec());
}

// --- Case 2: A1 host-supplied record_id ---------------------------------------
#[test]
fn a2_host_supplied_record_id_is_opaque_and_drives_receipt_id() {
    let pic = PocketIc::new();
    let cid = install(&pic);
    let caller = Principal::from_slice(&[0xB2, 0xB2, 0xB2, 0xB2]);
    // Arbitrary opaque bytes that are NOT the caller principal.
    let custom: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF, 0x7F, 0x01];

    let reply = pic
        .update_call(
            cid,
            caller,
            "h_execute_deletion_with_record_id",
            Encode!(&custom).unwrap(),
        )
        .expect("update_call failed");
    let receipt_id = Decode!(&reply, Result<Vec<u8>, String>)
        .unwrap()
        .expect("execute_deletion_with_record_id returned Err");

    let q = pic
        .query_call(
            cid,
            Principal::anonymous(),
            "h_get_receipt_fields",
            Encode!(&receipt_id).unwrap(),
        )
        .expect("query_call failed");
    let dto = Decode!(&q, Option<ReceiptFieldsDto>).unwrap().unwrap();

    // Stored verbatim — not parsed, validated, or transformed.
    assert_eq!(dto.record_id, custom);
    // Host value overrode the caller-derived value.
    assert_ne!(dto.record_id, caller.as_slice().to_vec());
    // It binds receipt_id per the live formula (record_id IS in the v3 preimage).
    let expected =
        zombie_core::receipt::compute_receipt_id(&dto.canister_id, &custom, dto.deletion_seq);
    assert_eq!(dto.receipt_id, expected.to_vec());
}

// --- Case 3: default-path controller guard, two cells -------------------------
#[test]
fn default_path_controller_guard_two_cells() {
    let pic = PocketIc::new();
    let noncontroller = Principal::from_slice(&[2, 2, 2, 2, 2]); // != anonymous controller

    // Cell A: pending receipt + non-controller -> NotController
    let cid = install(&pic);
    let r = pic
        .update_call(
            cid,
            Principal::anonymous(),
            "h_execute_deletion",
            Encode!().unwrap(),
        )
        .expect("update_call failed");
    let rid = Decode!(&r, Result<Vec<u8>, String>).unwrap().expect("Err");
    let out = pic
        .update_call(
            cid,
            noncontroller,
            "h_finalize_receipt",
            Encode!(&rid, &Vec::<u8>::new()).unwrap(),
        )
        .expect("update_call failed");
    assert_eq!(
        Decode!(&out, String).unwrap(),
        "NotController",
        "non-controller WITH a pending receipt must get NotController"
    );

    // Cell B: no pending receipt + non-controller -> NoPendingReceipt
    // (lock-check precedes the controller guard — the ordering under test)
    let cid2 = install(&pic);
    let dummy = vec![0u8; 32];
    let out2 = pic
        .update_call(
            cid2,
            noncontroller,
            "h_finalize_receipt",
            Encode!(&dummy, &Vec::<u8>::new()).unwrap(),
        )
        .expect("update_call failed");
    assert_eq!(
        Decode!(&out2, String).unwrap(),
        "NoPendingReceipt",
        "non-controller with NO pending receipt must get NoPendingReceipt"
    );
}
