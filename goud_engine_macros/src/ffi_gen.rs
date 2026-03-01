//! FFI wrapper code generation.
//!
//! This module generates `#[no_mangle] pub extern "C" fn` wrappers for
//! annotated SDK methods. Each generated function:
//!
//! 1. Takes a `GoudContextId` as its first parameter (for methods with self)
//! 2. Converts FFI types to Rust types
//! 3. Calls the original Rust method
//! 4. Converts the return value back to FFI types
//! 5. Includes proper null checks and SAFETY comments

use crate::codegen_helpers::{
    determine_receiver, extract_param_name, generate_param_conversion, generate_return_handling,
    generate_vec2_reconstruction, manifest_return_info, ReceiverKind,
};
use crate::manifest::{SdkMethod, SdkParam};
use crate::type_mapping::{map_return_type, map_type};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{FnArg, ImplItemFn};

/// Result of generating an FFI wrapper for a single method.
pub struct GeneratedFfi {
    /// The generated FFI function tokens.
    pub tokens: TokenStream,

    /// Manifest metadata for this method.
    pub manifest: SdkMethod,
}

/// Generates an FFI wrapper function for a single method.
///
/// # Arguments
///
/// * `method` - The parsed method from the impl block
/// * `module_name` - The module name from `#[goud_api(module = "...")]`
/// * `self_type` - The type that `self` refers to (e.g., `GoudGame`)
/// * `name_override` - Optional custom name for the FFI function name portion
///   (e.g., `Some("key_pressed")` produces `goud_input_key_pressed` instead
///   of `goud_input_is_key_pressed`)
pub fn generate_ffi_wrapper(
    method: &ImplItemFn,
    module_name: &str,
    self_type: &str,
    name_override: Option<&str>,
) -> Option<GeneratedFfi> {
    // Skip non-public methods
    if !matches!(method.vis, syn::Visibility::Public(_)) {
        return None;
    }

    let method_name = method.sig.ident.to_string();
    let ffi_method_name = name_override.unwrap_or(&method_name);
    let ffi_name = format!("goud_{}_{}", module_name, ffi_method_name);
    let ffi_ident = Ident::new(&ffi_name, Span::call_site());

    // Determine receiver kind
    let receiver = determine_receiver(&method.sig.inputs);

    // Collect non-self parameters
    let params: Vec<&FnArg> = method
        .sig
        .inputs
        .iter()
        .filter(|arg| !matches!(arg, FnArg::Receiver(_)))
        .collect();

    // Map return type
    let return_info = map_return_type(&method.sig.output);

    // Build FFI parameter list
    let mut ffi_params = Vec::new();
    let mut param_conversions = Vec::new();
    let mut null_checks = Vec::new();
    let mut needs_unsafe = false;
    let mut manifest_params = Vec::new();

    // Add context_id parameter for methods with self
    if receiver != ReceiverKind::None {
        ffi_params.push(quote! {
            context_id: crate::core::context_registry::GoudContextId
        });
    }

    // Process each non-self parameter
    for param in &params {
        if let FnArg::Typed(pat_type) = param {
            process_param(
                pat_type,
                &mut ffi_params,
                &mut param_conversions,
                &mut null_checks,
                &mut needs_unsafe,
                &mut manifest_params,
            );
        }
    }

    // Build out-params and return type based on return mapping
    let mut ffi_out_params_tokens = Vec::new();
    let mut manifest_out_params = Vec::new();
    let (ffi_return_type, body) = generate_return_handling(
        &return_info,
        &method.sig.ident,
        receiver,
        &param_conversions,
        &null_checks,
        &params,
        &mut ffi_out_params_tokens,
        &mut manifest_out_params,
        &mut needs_unsafe,
        self_type,
    );

    // Combine all FFI params (regular + out-params)
    let all_ffi_params: Vec<TokenStream> = ffi_params
        .into_iter()
        .chain(ffi_out_params_tokens)
        .collect();

    // Generate the function
    let tokens = if needs_unsafe {
        quote! {
            #[no_mangle]
            pub unsafe extern "C" fn #ffi_ident(
                #(#all_ffi_params),*
            ) #ffi_return_type {
                #body
            }
        }
    } else {
        quote! {
            #[no_mangle]
            pub extern "C" fn #ffi_ident(
                #(#all_ffi_params),*
            ) #ffi_return_type {
                #body
            }
        }
    };

    // Build manifest entry
    let (manifest_return_type, ffi_return_type_name) =
        manifest_return_info(&method.sig.output, &return_info);

    let manifest = SdkMethod {
        name: method_name,
        ffi_name,
        receiver: match receiver {
            ReceiverKind::Ref => "ref".to_string(),
            ReceiverKind::Mut => "mut".to_string(),
            ReceiverKind::None => "none".to_string(),
        },
        params: manifest_params,
        return_type: manifest_return_type,
        ffi_return_type: ffi_return_type_name,
        ffi_out_params: manifest_out_params,
    };

    Some(GeneratedFfi { tokens, manifest })
}

/// Processes a single typed parameter, adding FFI params, conversions, and
/// null checks as needed.
fn process_param(
    pat_type: &syn::PatType,
    ffi_params: &mut Vec<TokenStream>,
    param_conversions: &mut Vec<TokenStream>,
    null_checks: &mut Vec<TokenStream>,
    needs_unsafe: &mut bool,
    manifest_params: &mut Vec<SdkParam>,
) {
    let param_name = extract_param_name(&pat_type.pat);
    let type_info = map_type(&pat_type.ty);

    if type_info.needs_unsafe {
        *needs_unsafe = true;
    }

    let mut ffi_sub_params = Vec::new();

    if type_info.ffi_params.len() == 1 && type_info.ffi_params[0].name_suffix.is_empty() {
        process_simple_param(
            &param_name,
            &pat_type.ty,
            &type_info,
            ffi_params,
            param_conversions,
            null_checks,
            &mut ffi_sub_params,
        );
    } else {
        process_flattened_param(
            &param_name,
            &type_info,
            ffi_params,
            param_conversions,
            &mut ffi_sub_params,
        );
    }

    manifest_params.push(SdkParam {
        name: param_name,
        param_type: type_info.manifest_type_name,
        ffi_params: if ffi_sub_params.len() > 1 {
            ffi_sub_params
        } else {
            vec![]
        },
    });
}

/// Processes a simple (non-flattened) parameter.
fn process_simple_param(
    param_name: &str,
    ty: &syn::Type,
    type_info: &crate::type_mapping::FfiTypeInfo,
    ffi_params: &mut Vec<TokenStream>,
    param_conversions: &mut Vec<TokenStream>,
    null_checks: &mut Vec<TokenStream>,
    ffi_sub_params: &mut Vec<SdkParam>,
) {
    let ffi_type = &type_info.ffi_params[0].ffi_type;
    let p_ident = format_ident!("{}", param_name);
    ffi_params.push(quote! { #p_ident: #ffi_type });

    if let Some(conv) = generate_param_conversion(param_name, ty, &type_info.manifest_type_name) {
        param_conversions.push(conv);
    }

    if type_info.manifest_type_name == "&str" {
        null_checks.push(quote! {
            if #p_ident.is_null() {
                return Default::default();
            }
        });
    }

    ffi_sub_params.push(SdkParam {
        name: param_name.to_string(),
        param_type: type_info.ffi_params[0].type_name.clone(),
        ffi_params: vec![],
    });
}

/// Processes a flattened parameter (e.g., Vec2 -> _x, _y).
fn process_flattened_param(
    param_name: &str,
    type_info: &crate::type_mapping::FfiTypeInfo,
    ffi_params: &mut Vec<TokenStream>,
    param_conversions: &mut Vec<TokenStream>,
    ffi_sub_params: &mut Vec<SdkParam>,
) {
    for fp in &type_info.ffi_params {
        let p_ident = format_ident!("{}{}", param_name, fp.name_suffix);
        let ffi_type = &fp.ffi_type;
        ffi_params.push(quote! { #p_ident: #ffi_type });

        ffi_sub_params.push(SdkParam {
            name: format!("{}{}", param_name, fp.name_suffix),
            param_type: fp.type_name.clone(),
            ffi_params: vec![],
        });
    }

    param_conversions.push(generate_vec2_reconstruction(param_name));
}
