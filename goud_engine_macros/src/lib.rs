//! # GoudEngine Proc-Macro Crate
//!
//! Provides the `#[goud_api]` attribute macro for auto-generating FFI
//! wrappers from annotated `impl` blocks. This eliminates the need to
//! hand-write boilerplate FFI functions for each SDK method.
//!
//! ## Usage
//!
//! ```rust,ignore
//! #[goud_api(module = "window")]
//! impl GoudGame {
//!     pub fn should_close(&self) -> bool { ... }
//!     pub fn poll_events(&mut self) -> GoudResult<f32> { ... }
//! }
//! ```
//!
//! This generates:
//! - `goud_window_should_close(ctx: GoudContextId) -> bool`
//! - `goud_window_poll_events(ctx: GoudContextId, out: *mut f32) -> GoudResult`
//!
//! ## Attributes
//!
//! - `#[goud_api(module = "name")]` - Required. Sets the module prefix.
//! - `#[goud_api(module = "name", feature = "native")]` - Optional feature gate.
//! - `#[goud_api(skip)]` on a method - Excludes it from FFI generation.
//! - `#[goud_api(name = "custom")]` on a method - Overrides the FFI name portion.

mod codegen_helpers;
mod ffi_gen;
mod manifest;
mod type_mapping;

use ffi_gen::generate_ffi_wrapper;
use manifest::{generate_manifest_json, manifest_const_name, SdkModule};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ItemImpl, Meta};

/// The `#[goud_api]` attribute macro for auto-generating FFI wrappers.
///
/// Place on an `impl` block to generate `#[no_mangle] pub extern "C" fn`
/// wrappers for each public method.
///
/// # Attributes
///
/// - `module = "name"` (required): Sets the FFI function name prefix.
///   Methods on `impl Foo` with `module = "window"` produce functions
///   named `goud_window_<method_name>`.
///
/// - `feature = "native"` (optional): Wraps generated FFI functions in
///   `#[cfg(feature = "...")]`.
///
/// # Method-Level Attributes
///
/// - `#[goud_api(skip)]`: Exclude a method from FFI generation. Use this
///   for methods that are too complex for automatic wrapping (generics,
///   closures, builders, etc.).
///
/// # Example
///
/// ```rust,ignore
/// use goud_engine_macros::goud_api;
///
/// #[goud_api(module = "window")]
/// impl GoudGame {
///     pub fn should_close(&self) -> bool {
///         // implementation
///         false
///     }
///
///     #[goud_api(skip)]
///     pub fn run<F: FnMut()>(&mut self, callback: F) {
///         // Too complex for auto-gen, skipped
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn goud_api(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as GoudApiAttrs);
    let input = parse_macro_input!(item as ItemImpl);

    match expand_goud_api(attrs, input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Parsed attributes from `#[goud_api(module = "...", feature = "...")]`.
struct GoudApiAttrs {
    module: String,
    feature: Option<String>,
}

impl syn::parse::Parse for GoudApiAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut module = None;
        let mut feature = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let value: syn::LitStr = input.parse()?;

            match ident.to_string().as_str() {
                "module" => module = Some(value.value()),
                "feature" => feature = Some(value.value()),
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("Unknown goud_api attribute: {}", other),
                    ));
                }
            }

            // Consume optional comma
            if input.peek(syn::Token![,]) {
                let _comma: syn::Token![,] = input.parse()?;
            }
        }

        let module = module.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "goud_api requires `module = \"...\"` attribute",
            )
        })?;

        Ok(GoudApiAttrs { module, feature })
    }
}

/// Core expansion logic for `#[goud_api]`.
fn expand_goud_api(attrs: GoudApiAttrs, input: ItemImpl) -> syn::Result<TokenStream2> {
    let module_name = &attrs.module;

    // Extract the self type name for the impl block
    let self_type = extract_self_type(&input)?;

    // Collect generated FFI wrappers
    let mut ffi_functions = Vec::new();
    let mut manifest_methods = Vec::new();

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            // Check for #[goud_api(skip)]
            if has_skip_attr(method) {
                continue;
            }

            // Skip private methods
            if !matches!(method.vis, syn::Visibility::Public(_)) {
                continue;
            }

            // Check for #[goud_api(name = "...")] override
            let name_override = extract_name_attr(method);

            if let Some(generated) =
                generate_ffi_wrapper(method, module_name, &self_type, name_override.as_deref())
            {
                ffi_functions.push(generated.tokens);
                manifest_methods.push(generated.manifest);
            }
        }
    }

    // Build manifest module data
    let sdk_module = SdkModule {
        name: module_name.clone(),
        feature: attrs.feature.clone(),
        methods: manifest_methods,
        functions: vec![],
    };

    let manifest_json = generate_manifest_json(&sdk_module);
    let const_name = syn::Ident::new(
        &manifest_const_name(module_name),
        proc_macro2::Span::call_site(),
    );

    // Generate the hidden manifest const
    // `#[used]` prevents the linker from discarding the const even if it appears
    // unused at the Rust level; external tooling (build.rs / codegen) scans the
    // compiled binary or source for `GOUD_API_MANIFEST_*` prefixed symbols.
    let manifest_const = quote! {
        #[doc(hidden)]
        #[used]
        static #const_name: &str = #manifest_json;
    };

    // Optionally wrap FFI functions in cfg(feature)
    let ffi_block = if let Some(ref feature) = attrs.feature {
        let feature_ident = syn::LitStr::new(feature, proc_macro2::Span::call_site());
        quote! {
            #[cfg(feature = #feature_ident)]
            #[doc(hidden)]
            pub mod __goud_generated_ffi {
                use super::*;
                #(#ffi_functions)*
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            pub mod __goud_generated_ffi {
                use super::*;
                #(#ffi_functions)*
            }
        }
    };

    // Strip #[goud_api(...)] attributes from methods before emitting the
    // original impl block, so the compiler doesn't complain about unknown
    // attributes.
    let mut cleaned_input = input.clone();
    for item in &mut cleaned_input.items {
        if let ImplItem::Fn(method) = item {
            method
                .attrs
                .retain(|attr| !attr.path().is_ident("goud_api"));
        }
    }

    // Output: cleaned impl block + generated FFI + manifest const
    Ok(quote! {
        #cleaned_input
        #ffi_block
        #manifest_const
    })
}

/// Extracts the type name from an impl block (e.g., "GoudGame" from `impl GoudGame`).
fn extract_self_type(input: &ItemImpl) -> syn::Result<String> {
    if let syn::Type::Path(type_path) = &*input.self_ty {
        if let Some(segment) = type_path.path.segments.last() {
            return Ok(segment.ident.to_string());
        }
    }
    Err(syn::Error::new_spanned(
        &input.self_ty,
        "goud_api: cannot determine self type from impl block",
    ))
}

/// Checks if a method has `#[goud_api(skip)]`.
fn has_skip_attr(method: &syn::ImplItemFn) -> bool {
    for attr in &method.attrs {
        if attr.path().is_ident("goud_api") {
            if let Ok(Meta::List(meta_list)) = attr.parse_args::<Meta>() {
                if meta_list.path.is_ident("skip") {
                    return true;
                }
            }
            // Also handle the simple case: #[goud_api(skip)]
            if let Ok(ident) = attr.parse_args::<syn::Ident>() {
                if ident == "skip" {
                    return true;
                }
            }
        }
    }
    false
}

/// Extracts the `name` override from `#[goud_api(name = "custom_name")]`.
///
/// Returns `None` if no name override is present.
fn extract_name_attr(method: &syn::ImplItemFn) -> Option<String> {
    for attr in &method.attrs {
        if attr.path().is_ident("goud_api") {
            // Parse as a list of name-value pairs: name = "..."
            if let Ok(name_value) = attr.parse_args_with(|input: syn::parse::ParseStream| {
                let ident: syn::Ident = input.parse()?;
                if ident != "name" {
                    return Err(syn::Error::new(ident.span(), "expected `name`"));
                }
                let _eq: syn::Token![=] = input.parse()?;
                let value: syn::LitStr = input.parse()?;
                Ok(value.value())
            }) {
                return Some(name_value);
            }
        }
    }
    None
}
