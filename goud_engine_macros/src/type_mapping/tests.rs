//! Tests for the type mapping module.

#[cfg(test)]
use syn::Type;

#[cfg(test)]
use crate::type_mapping::{map_type, FfiReturn};

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
