//! Private helper functions for type analysis.
//!
//! These utilities inspect `syn::Type` values to identify and decompose
//! types at the FFI boundary.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

use super::types::{FfiParam, FfiReturn, FfiTypeInfo};

/// Identifies whether a type path matches a known type name.
pub(super) fn type_path_matches(ty: &Type, name: &str) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == name;
        }
    }
    false
}

/// Extracts the inner type from a generic type like `GoudResult<T>` or `Option<T>`.
pub(super) fn extract_generic_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                    return Some(inner);
                }
            }
        }
    }
    None
}

/// Extracts types from a tuple type like `(f32, f32)`.
pub(super) fn extract_tuple_types(ty: &Type) -> Option<Vec<&Type>> {
    if let Type::Tuple(tuple) = ty {
        Some(tuple.elems.iter().collect())
    } else {
        None
    }
}

/// Gets a simple type name string from a `syn::Type`.
pub(super) fn simple_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(p) => {
            if let Some(seg) = p.path.segments.last() {
                let ident = seg.ident.to_string();
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    let inner: Vec<String> = args
                        .args
                        .iter()
                        .filter_map(|arg| {
                            if let syn::GenericArgument::Type(t) = arg {
                                Some(simple_type_name(t))
                            } else {
                                None
                            }
                        })
                        .collect();
                    format!("{}<{}>", ident, inner.join(", "))
                } else {
                    ident
                }
            } else {
                "unknown".to_string()
            }
        }
        Type::Tuple(t) => {
            let inner: Vec<String> = t.elems.iter().map(simple_type_name).collect();
            format!("({})", inner.join(", "))
        }
        Type::Reference(r) => {
            let inner = simple_type_name(&r.elem);
            if r.mutability.is_some() {
                format!("&mut {}", inner)
            } else {
                format!("&{}", inner)
            }
        }
        _ => "unknown".to_string(),
    }
}

/// Creates an `FfiTypeInfo` for a primitive pass-through type.
pub(super) fn primitive_passthrough(name: &str, ffi_type: TokenStream) -> FfiTypeInfo {
    FfiTypeInfo {
        ffi_params: vec![FfiParam {
            name_suffix: String::new(),
            ffi_type: ffi_type.clone(),
            type_name: name.to_string(),
        }],
        ffi_return: FfiReturn::Direct(ffi_type, name.to_string()),
        needs_unsafe: false,
        manifest_type_name: name.to_string(),
    }
}

/// Primitive type names that map directly to the same FFI type.
pub(super) const PRIMITIVES: &[&str] = &["bool", "f32", "i32", "u8", "u32", "u64", "usize"];

/// Resolves a primitive name to its `TokenStream` type token.
pub(super) fn primitive_token(prim: &str) -> TokenStream {
    match prim {
        "bool" => quote! { bool },
        "f32" => quote! { f32 },
        "i32" => quote! { i32 },
        "u8" => quote! { u8 },
        "u32" => quote! { u32 },
        "u64" => quote! { u64 },
        "usize" => quote! { usize },
        _ => unreachable!(),
    }
}
