use super::params::{build_call_args, build_method_call, build_method_call_inner};
use super::receiver::{registry_access_for, ReceiverKind};
use crate::manifest::{SdkParam, SdkReturnType};
use crate::type_mapping::{map_type, FfiReturn};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{FnArg, ReturnType};

/// Generates the return type token and function body for an FFI wrapper.
#[allow(clippy::too_many_arguments)]
pub fn generate_return_handling(
    ri: &FfiReturn,
    mi: &Ident,
    recv: ReceiverKind,
    pc: &[TokenStream],
    nc: &[TokenStream],
    params: &[&FnArg],
    opt: &mut Vec<TokenStream>,
    mop: &mut Vec<SdkParam>,
    nu: &mut bool,
    self_type: &str,
) -> (TokenStream, TokenStream) {
    let ca = build_call_args(params);
    match ri {
        FfiReturn::Void => handle_void(mi, recv, pc, nc, &ca, self_type),
        FfiReturn::Direct(ft, _) => handle_direct(ft, mi, recv, pc, nc, &ca, self_type),
        FfiReturn::ResultWithOutParams {
            out_params,
            inner_type_name,
        } => handle_result(
            out_params,
            inner_type_name,
            mi,
            recv,
            pc,
            nc,
            &ca,
            opt,
            mop,
            nu,
            self_type,
        ),
        FfiReturn::OptionWithOutParam { out_params, .. } => {
            handle_option(out_params, mi, recv, pc, nc, &ca, opt, mop, nu, self_type)
        }
        FfiReturn::TupleOutParams { out_params } => {
            handle_tuple(out_params, mi, recv, pc, nc, &ca, opt, mop, nu, self_type)
        }
    }
}

fn handle_void(
    method_ident: &Ident,
    receiver: ReceiverKind,
    param_conversions: &[TokenStream],
    null_checks: &[TokenStream],
    call_args: &[Ident],
    self_type: &str,
) -> (TokenStream, TokenStream) {
    let call = build_method_call(method_ident, receiver, call_args, self_type);
    (
        quote! {},
        quote! { #(#null_checks)* #(#param_conversions)* #call },
    )
}

fn handle_direct(
    ffi_type: &TokenStream,
    method_ident: &Ident,
    receiver: ReceiverKind,
    param_conversions: &[TokenStream],
    null_checks: &[TokenStream],
    call_args: &[Ident],
    self_type: &str,
) -> (TokenStream, TokenStream) {
    let call = build_method_call(method_ident, receiver, call_args, self_type);
    (
        quote! { -> #ffi_type },
        quote! { #(#null_checks)* #(#param_conversions)* #call },
    )
}

#[allow(clippy::too_many_arguments)]
fn handle_result(
    out_params: &[crate::type_mapping::FfiParam],
    _inner_type_name: &str,
    method_ident: &Ident,
    receiver: ReceiverKind,
    param_conversions: &[TokenStream],
    null_checks: &[TokenStream],
    call_args: &[Ident],
    out_param_tokens: &mut Vec<TokenStream>,
    manifest_out_params: &mut Vec<SdkParam>,
    needs_unsafe: &mut bool,
    self_type: &str,
) -> (TokenStream, TokenStream) {
    *needs_unsafe = !out_params.is_empty();

    for op in out_params.iter() {
        let out_name = if out_params.len() == 1 {
            format_ident!("out_value")
        } else {
            format_ident!("out_value{}", op.name_suffix)
        };
        let ffi_type = &op.ffi_type;
        out_param_tokens.push(quote! { #out_name: #ffi_type });
        manifest_out_params.push(SdkParam {
            name: out_name.to_string(),
            param_type: op.type_name.clone(),
            ffi_params: vec![],
        });
    }

    let call = build_method_call(method_ident, receiver, call_args, self_type);
    let body = if out_params.is_empty() {
        build_result_void_body(
            receiver,
            method_ident,
            param_conversions,
            null_checks,
            call_args,
            &call,
        )
    } else {
        build_result_value_body(
            receiver,
            method_ident,
            param_conversions,
            null_checks,
            call_args,
            &call,
        )
    };

    (quote! { -> crate::core::types::GoudResult }, body)
}

fn goud_err() -> TokenStream {
    quote! { crate::core::types::GoudResult::err(crate::core::error::ERR_INVALID_CONTEXT) }
}

fn with_registry_result(
    receiver: ReceiverKind,
    method_ident: &Ident,
    call_args: &[Ident],
    preamble: &TokenStream,
    match_body: impl FnOnce(TokenStream) -> TokenStream,
) -> TokenStream {
    let err = goud_err();
    let inner = build_method_call_inner(method_ident, call_args);
    let access = registry_access_for(receiver);
    let matched = match_body(quote! { context.#inner });
    quote! {
        #preamble
        if context_id.is_invalid() { return #err; }
        let mut registry = match crate::core::context_registry::get_context_registry().lock() {
            Ok(r) => r, Err(_) => return #err,
        };
        #access
        #matched
    }
}

fn build_result_void_body(
    receiver: ReceiverKind,
    method_ident: &Ident,
    param_conversions: &[TokenStream],
    null_checks: &[TokenStream],
    call_args: &[Ident],
    call: &TokenStream,
) -> TokenStream {
    let preamble = quote! { #(#null_checks)* #(#param_conversions)* };
    let ok_arm = |expr: TokenStream| {
        quote! {
            match #expr {
                Ok(()) => crate::core::types::GoudResult::ok(),
                Err(e) => crate::core::types::GoudResult::err(e.error_code()),
            }
        }
    };
    match receiver {
        ReceiverKind::None => {
            let m = ok_arm(call.clone());
            quote! { #preamble #m }
        }
        _ => with_registry_result(receiver, method_ident, call_args, &preamble, ok_arm),
    }
}

fn build_result_value_body(
    receiver: ReceiverKind,
    method_ident: &Ident,
    param_conversions: &[TokenStream],
    null_checks: &[TokenStream],
    call_args: &[Ident],
    call: &TokenStream,
) -> TokenStream {
    let out_name = format_ident!("out_value");
    let err = goud_err();
    let null_check = quote! { if #out_name.is_null() { return #err; } };
    let preamble = quote! { #null_check #(#null_checks)* #(#param_conversions)* };
    let ok_arm = |expr: TokenStream| {
        quote! {
            match #expr {
                Ok(val) => { // SAFETY: out_value checked non-null above.
                    *#out_name = val; crate::core::types::GoudResult::ok()
                }
                Err(e) => crate::core::types::GoudResult::err(e.error_code()),
            }
        }
    };
    match receiver {
        ReceiverKind::None => {
            let m = ok_arm(call.clone());
            quote! { #preamble #m }
        }
        _ => with_registry_result(receiver, method_ident, call_args, &preamble, ok_arm),
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_option(
    out_params: &[crate::type_mapping::FfiParam],
    method_ident: &Ident,
    receiver: ReceiverKind,
    param_conversions: &[TokenStream],
    null_checks: &[TokenStream],
    call_args: &[Ident],
    out_param_tokens: &mut Vec<TokenStream>,
    manifest_out_params: &mut Vec<SdkParam>,
    needs_unsafe: &mut bool,
    self_type: &str,
) -> (TokenStream, TokenStream) {
    *needs_unsafe = true;

    for op in out_params {
        let out_name = format_ident!("out_contact");
        let ffi_type = &op.ffi_type;
        out_param_tokens.push(quote! { #out_name: #ffi_type });
        manifest_out_params.push(SdkParam {
            name: "out_contact".to_string(),
            param_type: op.type_name.clone(),
            ffi_params: vec![],
        });
    }

    let call = build_method_call(method_ident, receiver, call_args, self_type);
    let body = quote! {
        #(#null_checks)* #(#param_conversions)*
        match #call {
            Some(contact) => {
                if !out_contact.is_null() {
                    // SAFETY: Caller guarantees pointer is valid if non-null.
                    *out_contact = contact.into();
                }
                true
            }
            None => false,
        }
    };

    (quote! { -> bool }, body)
}

#[allow(clippy::too_many_arguments)]
fn handle_tuple(
    out_params: &[crate::type_mapping::FfiParam],
    method_ident: &Ident,
    receiver: ReceiverKind,
    param_conversions: &[TokenStream],
    null_checks: &[TokenStream],
    call_args: &[Ident],
    out_param_tokens: &mut Vec<TokenStream>,
    manifest_out_params: &mut Vec<SdkParam>,
    needs_unsafe: &mut bool,
    self_type: &str,
) -> (TokenStream, TokenStream) {
    *needs_unsafe = true;

    for (i, op) in out_params.iter().enumerate() {
        let out_name = format_ident!("out_{}", i);
        let ffi_type = &op.ffi_type;
        out_param_tokens.push(quote! { #out_name: #ffi_type });
        manifest_out_params.push(SdkParam {
            name: format!("out_{}", i),
            param_type: op.type_name.clone(),
            ffi_params: vec![],
        });
    }

    let out_null_checks: Vec<TokenStream> = (0..out_params.len())
        .map(|i| {
            let n = format_ident!("out_{}", i);
            quote! { if #n.is_null() { return false; } }
        })
        .collect();

    let out_writes: Vec<TokenStream> = (0..out_params.len())
        .map(|i| {
            let n = format_ident!("out_{}", i);
            let idx = syn::Index::from(i);
            quote! { // SAFETY: We checked that pointer is non-null above.
            *#n = result.#idx; }
        })
        .collect();

    let call = build_method_call(method_ident, receiver, call_args, self_type);
    let body = match receiver {
        ReceiverKind::None => quote! {
            #(#out_null_checks)* #(#null_checks)* #(#param_conversions)*
            let result = #call;
            #(#out_writes)*
            true
        },
        _ => {
            let inner = build_method_call_inner(method_ident, call_args);
            let access = registry_access_for(receiver);
            quote! {
                #(#out_null_checks)* #(#null_checks)* #(#param_conversions)*
                if context_id.is_invalid() { return false; }
                let mut registry = match crate::core::context_registry::get_context_registry().lock() {
                    Ok(r) => r, Err(_) => return false,
                };
                #access
                let result = context.#inner;
                #(#out_writes)*
                true
            }
        }
    };

    (quote! { -> bool }, body)
}

/// Builds manifest return type info from the Rust return type.
pub fn manifest_return_info(
    return_type: &ReturnType,
    ffi_return: &FfiReturn,
) -> (SdkReturnType, String) {
    match ffi_return {
        FfiReturn::Void => (SdkReturnType::simple("()"), "()".to_string()),
        FfiReturn::Direct(_, name) => (SdkReturnType::simple(name.clone()), name.clone()),
        FfiReturn::ResultWithOutParams {
            inner_type_name, ..
        } => (
            SdkReturnType::result(inner_type_name.clone()),
            "GoudResult".to_string(),
        ),
        FfiReturn::OptionWithOutParam {
            inner_type_name, ..
        } => (
            SdkReturnType::option(inner_type_name.clone()),
            "bool".to_string(),
        ),
        FfiReturn::TupleOutParams { .. } => {
            let type_str = match return_type {
                ReturnType::Type(_, ty) => map_type(ty).manifest_type_name,
                _ => "()".to_string(),
            };
            (SdkReturnType::simple(type_str), "bool".to_string())
        }
    }
}
