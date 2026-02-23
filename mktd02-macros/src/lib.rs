//! # MKTd02 Procedural Macros
//!
//! ## `#[mktd_guard]`
//!
//! Injects tombstone + initialisation checks at the top of a function body.
//! Uses the `GuardError` trait from `mktd02::trait_def`.
//!
//! ### Usage
//!
//! ```rust,ignore
//! #[mktd_guard]
//! fn upsert_profile(data: ProfileData) -> Result<(), MyError> {
//!     // ... your logic here
//! }
//! ```
//!
//! Expands to:
//!
//! ```rust,ignore
//! fn upsert_profile(data: ProfileData) -> Result<(), MyError> {
//!     if !mktd02::is_initialised() {
//!         return Err(<_ as mktd02::GuardError>::not_initialised());
//!     }
//!     if mktd02::is_tombstoned() {
//!         return Err(<_ as mktd02::GuardError>::tombstone_violation());
//!     }
//!     // ... your logic here
//! }
//! ```
//!
//! The function must return `Result<T, E>` where `E: mktd02::GuardError`.
//!
//! **Non-macro alternative:** `mktd02::assert_can_write()` for functions
//! returning `()` or preferring trap semantics.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Attribute macro that injects MKTd02 tombstone and initialisation
/// guards at the top of a function body.
///
/// The annotated function must return `Result<T, E>` where
/// `E: mktd02::GuardError`.
#[proc_macro_attribute]
pub fn mktd_guard(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);

    let original_body = &func.block;

    let guarded_body = quote! {
        {
            if !mktd02::is_initialised() {
                return Err(<_ as mktd02::GuardError>::not_initialised());
            }
            if mktd02::is_tombstoned() {
                return Err(<_ as mktd02::GuardError>::tombstone_violation());
            }
            #original_body
        }
    };

    func.block = syn::parse2(guarded_body).expect("mktd_guard: failed to parse guarded body");

    TokenStream::from(quote! { #func })
}
