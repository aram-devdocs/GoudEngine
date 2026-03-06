//! Type definitions for FFI type mapping.
//!
//! Defines how Rust types are represented at the FFI boundary, including
//! parameter flattening and return-value strategies.

use proc_macro2::TokenStream;

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
