//! # MKTd02 Procedural Macros
//!
//! ## `#[mktd_guard]`
//!
//! Injects tombstone + initialisation checks at the top of a function body.
//! The function **must** return `Result<T, E>` where `E: mktd02::GuardError`.
//!
//! **Non-macro alternative:** `mktd02::assert_can_write()` for functions
//! returning `()` or preferring trap semantics.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type, PathArguments, GenericArgument};

/// Extract the error type `E` from a `Result<T, E>` return type.
/// Returns None if the return type is not a recognizable Result.
fn extract_result_error_type(ret: &ReturnType) -> Option<proc_macro2::TokenStream> {
    let ty = match ret {
        ReturnType::Type(_, ty) => ty.as_ref(),
        ReturnType::Default => return None,
    };

    // Walk through the type to find Result<T, E>
    if let Type::Path(type_path) = ty {
        let last_seg = type_path.path.segments.last()?;
        let ident = &last_seg.ident;

        // Check if it's "Result"
        if ident != "Result" {
            return None;
        }

        if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
            // Result<T, E> has two generic args; E is the second
            let mut iter = args.args.iter();
            let _t = iter.next()?; // skip T
            let e = iter.next()?;  // get E

            if let GenericArgument::Type(error_type) = e {
                return Some(quote! { #error_type });
            }
        }
    }

    None
}

/// Attribute macro that injects MKTd02 tombstone and initialisation
/// guards at the top of a function body.
///
/// The annotated function must return `Result<T, E>` where
/// `E: mktd02::GuardError`. If the return type cannot be parsed as
/// `Result`, a compile error is emitted.
#[proc_macro_attribute]
pub fn mktd_guard(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);

    let error_type = match extract_result_error_type(&func.sig.output) {
        Some(e) => e,
        None => {
            return TokenStream::from(syn::Error::new_spanned(
                &func.sig,
                "#[mktd_guard] requires a function returning Result<T, E> where E: mktd02::GuardError. \
                 For non-Result functions, use mktd02::assert_can_write() instead."
            ).to_compile_error());
        }
    };

    let original_body = &func.block;

    let guarded_body = quote! {
        {
            if !mktd02::is_initialised() {
                return Err(<#error_type as mktd02::GuardError>::not_initialised());
            }
            if mktd02::is_tombstoned() {
                return Err(<#error_type as mktd02::GuardError>::tombstone_violation());
            }
            #original_body
        }
    };

    func.block = syn::parse2(guarded_body).expect("mktd_guard: failed to parse guarded body");

    TokenStream::from(quote! { #func })
}
