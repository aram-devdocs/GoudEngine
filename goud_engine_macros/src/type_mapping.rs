//! Type mapping from SDK/Rust types to FFI-compatible types.
//!
//! This module defines how Rust types are converted to C-ABI-safe types
//! for generated FFI wrapper functions.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/// Information about how a Rust type maps to FFI.
#[derive(Debug, Clone)]
pub struct FfiTypeInfo {
    /// The FFI-compatible parameter type(s). For flattened types like Vec2,
    /// this produces multiple parameters.
    pub ffi_params: Vec<FfiParam>,

    /// The FFI return type, if this type is used as a return value.
    pub ffi_return: FfiReturn,

    /// Whether using this type requires the FFI function to be `unsafe`.
    pub needs_unsafe: bool,

    /// Human-readable name for manifest generation.
    pub manifest_type_name: String,
}

/// A single FFI parameter derived from a Rust type.
#[derive(Debug, Clone)]
pub struct FfiParam {
    /// Parameter name suffix (e.g., "_x", "_y" for Vec2 flattening).
    pub name_suffix: String,

    /// The FFI type for this parameter.
    pub ffi_type: TokenStream,

    /// Human-readable type name for manifest.
    pub type_name: String,
}

/// How a Rust return type maps to FFI.
#[derive(Debug, Clone)]
pub enum FfiReturn {
    /// Direct return (e.g., bool -> bool, f32 -> f32).
    Direct(TokenStream, String),

    /// Return via GoudResult with out-parameters for the value.
    ResultWithOutParams {
        out_params: Vec<FfiParam>,
        /// The conversion expression from the Ok value to writing out-params.
        /// Not stored as TokenStream since it depends on context.
        inner_type_name: String,
    },

    /// Return via bool with optional out-parameter (for Option types).
    OptionWithOutParam {
        out_params: Vec<FfiParam>,
        inner_type_name: String,
    },

    /// Void return (the Rust function returns ()).
    Void,

    /// Tuple return via multiple out-parameters.
    TupleOutParams { out_params: Vec<FfiParam> },
}

/// Identifies whether a type path matches a known type name.
fn type_path_matches(ty: &Type, name: &str) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == name;
        }
    }
    false
}

/// Extracts the inner type from a generic type like `GoudResult<T>` or `Option<T>`.
fn extract_generic_inner(ty: &Type) -> Option<&Type> {
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
fn extract_tuple_types(ty: &Type) -> Option<Vec<&Type>> {
    if let Type::Tuple(tuple) = ty {
        Some(tuple.elems.iter().collect())
    } else {
        None
    }
}

/// Gets a simple type name string from a syn::Type.
fn simple_type_name(ty: &Type) -> String {
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

/// Creates an FfiTypeInfo for a primitive pass-through type.
fn primitive_passthrough(name: &str, ffi_type: TokenStream) -> FfiTypeInfo {
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
const PRIMITIVES: &[&str] = &["bool", "f32", "i32", "u8", "u32", "u64", "usize"];

/// Maps a Rust type to its FFI representation.
///
/// This is the core type mapping logic used by the FFI code generator.
pub fn map_type(ty: &Type) -> FfiTypeInfo {
    let type_name = simple_type_name(ty);

    // Primitives: pass-through
    for prim in PRIMITIVES {
        if type_path_matches(ty, prim) {
            let ffi_type = match *prim {
                "bool" => quote! { bool },
                "f32" => quote! { f32 },
                "i32" => quote! { i32 },
                "u8" => quote! { u8 },
                "u32" => quote! { u32 },
                "u64" => quote! { u64 },
                "usize" => quote! { usize },
                _ => unreachable!(),
            };
            return primitive_passthrough(prim, ffi_type);
        }
    }

    // Vec2: flatten to x, y params
    if type_path_matches(ty, "Vec2") {
        return FfiTypeInfo {
            ffi_params: vec![
                FfiParam {
                    name_suffix: "_x".to_string(),
                    ffi_type: quote! { f32 },
                    type_name: "f32".to_string(),
                },
                FfiParam {
                    name_suffix: "_y".to_string(),
                    ffi_type: quote! { f32 },
                    type_name: "f32".to_string(),
                },
            ],
            ffi_return: FfiReturn::TupleOutParams {
                out_params: vec![
                    FfiParam {
                        name_suffix: "_x".to_string(),
                        ffi_type: quote! { *mut f32 },
                        type_name: "f32".to_string(),
                    },
                    FfiParam {
                        name_suffix: "_y".to_string(),
                        ffi_type: quote! { *mut f32 },
                        type_name: "f32".to_string(),
                    },
                ],
            },
            needs_unsafe: false,
            manifest_type_name: "Vec2".to_string(),
        };
    }

    // Entity: map to u64
    if type_path_matches(ty, "Entity") {
        return FfiTypeInfo {
            ffi_params: vec![FfiParam {
                name_suffix: String::new(),
                ffi_type: quote! { u64 },
                type_name: "u64".to_string(),
            }],
            ffi_return: FfiReturn::Direct(quote! { u64 }, "u64".to_string()),
            needs_unsafe: false,
            manifest_type_name: "Entity".to_string(),
        };
    }

    // &str: map to *const c_char (needs unsafe)
    if let Type::Reference(ref_type) = ty {
        if type_path_matches(&ref_type.elem, "str") {
            return FfiTypeInfo {
                ffi_params: vec![FfiParam {
                    name_suffix: String::new(),
                    ffi_type: quote! { *const std::os::raw::c_char },
                    type_name: "*const c_char".to_string(),
                }],
                ffi_return: FfiReturn::Direct(
                    quote! { *const std::os::raw::c_char },
                    "*const c_char".to_string(),
                ),
                needs_unsafe: true,
                manifest_type_name: "&str".to_string(),
            };
        }
    }

    // Key (glfw): map to i32
    if type_path_matches(ty, "Key") {
        return FfiTypeInfo {
            ffi_params: vec![FfiParam {
                name_suffix: String::new(),
                ffi_type: quote! { i32 },
                type_name: "i32".to_string(),
            }],
            ffi_return: FfiReturn::Direct(quote! { i32 }, "i32".to_string()),
            needs_unsafe: false,
            manifest_type_name: "Key".to_string(),
        };
    }

    // MouseButton: map to i32
    if type_path_matches(ty, "MouseButton") {
        return FfiTypeInfo {
            ffi_params: vec![FfiParam {
                name_suffix: String::new(),
                ffi_type: quote! { i32 },
                type_name: "i32".to_string(),
            }],
            ffi_return: FfiReturn::Direct(quote! { i32 }, "i32".to_string()),
            needs_unsafe: false,
            manifest_type_name: "MouseButton".to_string(),
        };
    }

    // GoudResult<T>: return GoudResult with out-params for T
    if type_path_matches(ty, "GoudResult") {
        if let Some(inner) = extract_generic_inner(ty) {
            // GoudResult<()> -> just return GoudResult
            if let Type::Tuple(tuple) = inner {
                if tuple.elems.is_empty() {
                    return FfiTypeInfo {
                        ffi_params: vec![],
                        ffi_return: FfiReturn::ResultWithOutParams {
                            out_params: vec![],
                            inner_type_name: "()".to_string(),
                        },
                        needs_unsafe: false,
                        manifest_type_name: "GoudResult<()>".to_string(),
                    };
                }
            }

            // GoudResult<primitive> -> out-param + GoudResult
            let inner_info = map_type(inner);
            let out_params: Vec<FfiParam> = inner_info
                .ffi_params
                .iter()
                .map(|p| FfiParam {
                    name_suffix: if p.name_suffix.is_empty() {
                        String::new()
                    } else {
                        p.name_suffix.clone()
                    },
                    ffi_type: {
                        let t = &p.ffi_type;
                        quote! { *mut #t }
                    },
                    type_name: p.type_name.clone(),
                })
                .collect();

            return FfiTypeInfo {
                ffi_params: vec![],
                ffi_return: FfiReturn::ResultWithOutParams {
                    out_params,
                    inner_type_name: inner_info.manifest_type_name.clone(),
                },
                needs_unsafe: true, // out-params need unsafe for pointer writes
                manifest_type_name: format!("GoudResult<{}>", inner_info.manifest_type_name),
            };
        }
    }

    // Option<Contact>: return bool with out-param
    if type_path_matches(ty, "Option") {
        if let Some(inner) = extract_generic_inner(ty) {
            if type_path_matches(inner, "Contact") {
                return FfiTypeInfo {
                    ffi_params: vec![],
                    ffi_return: FfiReturn::OptionWithOutParam {
                        out_params: vec![FfiParam {
                            name_suffix: String::new(),
                            ffi_type: quote! { *mut crate::core::types::GoudContact },
                            type_name: "GoudContact".to_string(),
                        }],
                        inner_type_name: "Contact".to_string(),
                    },
                    needs_unsafe: true,
                    manifest_type_name: "Option<Contact>".to_string(),
                };
            }
        }
    }

    // Tuple types: (f32, f32) -> out-params
    if let Some(types) = extract_tuple_types(ty) {
        let out_params: Vec<FfiParam> = types
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let inner = map_type(t);
                let suffix = format!("_{}", i);
                FfiParam {
                    name_suffix: suffix,
                    ffi_type: {
                        let ft = &inner.ffi_params[0].ffi_type;
                        quote! { *mut #ft }
                    },
                    type_name: inner.ffi_params[0].type_name.clone(),
                }
            })
            .collect();

        return FfiTypeInfo {
            ffi_params: vec![],
            ffi_return: FfiReturn::TupleOutParams {
                out_params: out_params.clone(),
            },
            needs_unsafe: true,
            manifest_type_name: type_name.clone(),
        };
    }

    // PrimitiveCreateInfo: already #[repr(C)], pass directly
    if type_path_matches(ty, "PrimitiveCreateInfo") {
        return FfiTypeInfo {
            ffi_params: vec![FfiParam {
                name_suffix: String::new(),
                ffi_type: quote! { crate::libs::graphics::renderer3d::PrimitiveCreateInfo },
                type_name: "PrimitiveCreateInfo".to_string(),
            }],
            ffi_return: FfiReturn::Direct(
                quote! { crate::libs::graphics::renderer3d::PrimitiveCreateInfo },
                "PrimitiveCreateInfo".to_string(),
            ),
            needs_unsafe: false,
            manifest_type_name: "PrimitiveCreateInfo".to_string(),
        };
    }

    // FFI repr(C) struct types: pass through directly as they are already C-compatible.
    // These types are defined in core/types.rs and core/context_registry.rs.
    const FFI_PASSTHROUGH_TYPES: &[(&str, &str)] = &[
        ("FfiTransform2D", "crate::core::types::FfiTransform2D"),
        ("FfiTransform2DBuilder", "crate::core::types::FfiTransform2DBuilder"),
        ("FfiMat3x3", "crate::core::types::FfiMat3x3"),
        ("FfiSprite", "crate::core::types::FfiSprite"),
        ("FfiSpriteBuilder", "crate::core::types::FfiSpriteBuilder"),
        ("FfiColor", "crate::core::types::FfiColor"),
        ("FfiRect", "crate::core::types::FfiRect"),
        ("FfiVec2", "crate::core::types::FfiVec2"),
        ("GoudContextId", "crate::core::context_registry::GoudContextId"),
        ("GoudEntityId", "crate::core::types::GoudEntityId"),
        ("GoudResult", "crate::core::types::GoudResult"),
        ("GoudRenderStats", "crate::ffi::renderer::GoudRenderStats"),
    ];

    for &(name, _path) in FFI_PASSTHROUGH_TYPES {
        if type_path_matches(ty, name) {
            // Use the type as-is (the SDK file imports it, so the short name works)
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            return FfiTypeInfo {
                ffi_params: vec![FfiParam {
                    name_suffix: String::new(),
                    ffi_type: quote! { #ident },
                    type_name: name.to_string(),
                }],
                ffi_return: FfiReturn::Direct(quote! { #ident }, name.to_string()),
                needs_unsafe: false,
                manifest_type_name: name.to_string(),
            };
        }
    }

    // Raw pointer types: *mut T and *const T pass through directly.
    // These are already FFI-compatible and need `unsafe` for the caller.
    if let Type::Ptr(ptr_type) = ty {
        let inner_name = simple_type_name(&ptr_type.elem);
        let inner_ty = &ptr_type.elem;
        let (ptr_kind, ffi_type) = if ptr_type.mutability.is_some() {
            ("*mut", quote! { *mut #inner_ty })
        } else {
            ("*const", quote! { *const #inner_ty })
        };
        let full_name = format!("{} {}", ptr_kind, inner_name);
        return FfiTypeInfo {
            ffi_params: vec![FfiParam {
                name_suffix: String::new(),
                ffi_type: ffi_type.clone(),
                type_name: full_name.clone(),
            }],
            ffi_return: FfiReturn::Direct(ffi_type, full_name.clone()),
            needs_unsafe: true,
            manifest_type_name: full_name,
        };
    }

    // Fallback: treat as opaque (will likely need manual handling)
    FfiTypeInfo {
        ffi_params: vec![FfiParam {
            name_suffix: String::new(),
            ffi_type: quote! { /* unsupported type */ },
            type_name: type_name.clone(),
        }],
        ffi_return: FfiReturn::Void,
        needs_unsafe: false,
        manifest_type_name: type_name,
    }
}

/// Maps a return type to FfiReturn info. Handles the unit type `()` specially.
pub fn map_return_type(ty: &syn::ReturnType) -> FfiReturn {
    match ty {
        syn::ReturnType::Default => FfiReturn::Void,
        syn::ReturnType::Type(_, ty) => {
            // Check for unit tuple
            if let Type::Tuple(tuple) = ty.as_ref() {
                if tuple.elems.is_empty() {
                    return FfiReturn::Void;
                }
            }
            map_type(ty).ffi_return
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_bool() {
        let ty: Type = syn::parse_str("bool").unwrap();
        let info = map_type(&ty);
        assert_eq!(info.manifest_type_name, "bool");
        assert!(!info.needs_unsafe);
        assert_eq!(info.ffi_params.len(), 1);
    }

    #[test]
    fn test_map_f32() {
        let ty: Type = syn::parse_str("f32").unwrap();
        let info = map_type(&ty);
        assert_eq!(info.manifest_type_name, "f32");
        assert!(!info.needs_unsafe);
    }

    #[test]
    fn test_map_vec2() {
        let ty: Type = syn::parse_str("Vec2").unwrap();
        let info = map_type(&ty);
        assert_eq!(info.manifest_type_name, "Vec2");
        assert_eq!(info.ffi_params.len(), 2);
        assert_eq!(info.ffi_params[0].name_suffix, "_x");
        assert_eq!(info.ffi_params[1].name_suffix, "_y");
    }

    #[test]
    fn test_map_entity() {
        let ty: Type = syn::parse_str("Entity").unwrap();
        let info = map_type(&ty);
        assert_eq!(info.manifest_type_name, "Entity");
        assert_eq!(info.ffi_params[0].type_name, "u64");
    }

    #[test]
    fn test_map_str_ref() {
        let ty: Type = syn::parse_str("&str").unwrap();
        let info = map_type(&ty);
        assert_eq!(info.manifest_type_name, "&str");
        assert!(info.needs_unsafe);
    }

    #[test]
    fn test_map_goud_result_unit() {
        let ty: Type = syn::parse_str("GoudResult<()>").unwrap();
        let info = map_type(&ty);
        assert_eq!(info.manifest_type_name, "GoudResult<()>");
        matches!(info.ffi_return, FfiReturn::ResultWithOutParams { .. });
    }

    #[test]
    fn test_map_goud_result_f32() {
        let ty: Type = syn::parse_str("GoudResult<f32>").unwrap();
        let info = map_type(&ty);
        assert_eq!(info.manifest_type_name, "GoudResult<f32>");
        assert!(info.needs_unsafe);
        if let FfiReturn::ResultWithOutParams { out_params, .. } = &info.ffi_return {
            assert_eq!(out_params.len(), 1);
        } else {
            panic!("Expected ResultWithOutParams");
        }
    }

    #[test]
    fn test_map_tuple() {
        let ty: Type = syn::parse_str("(f32, f32)").unwrap();
        let info = map_type(&ty);
        assert!(info.needs_unsafe);
        if let FfiReturn::TupleOutParams { out_params } = &info.ffi_return {
            assert_eq!(out_params.len(), 2);
        } else {
            panic!("Expected TupleOutParams");
        }
    }
}
