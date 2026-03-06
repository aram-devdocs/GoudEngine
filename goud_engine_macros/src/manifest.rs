//! Manifest metadata collection for SDK codegen.
//!
//! This module defines serializable structs that capture method metadata
//! during proc-macro expansion. The metadata is emitted as `const` strings
//! containing JSON, which can be collected by build.rs or a codegen script
//! to produce the unified `sdk_api.json` manifest.

use serde::{Deserialize, Serialize};

/// A single module's worth of annotated methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkModule {
    /// Module name from the `#[goud_api(module = "...")]` attribute.
    pub name: String,

    /// Optional feature gate (e.g., "native").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature: Option<String>,

    /// Methods in this module (from impl blocks).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub methods: Vec<SdkMethod>,

    /// Free functions in this module (not on an impl block).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub functions: Vec<SdkMethod>,
}

/// A single SDK method or function with its FFI mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkMethod {
    /// Original Rust method name (e.g., "should_close").
    pub name: String,

    /// Generated FFI function name (e.g., "goud_window_should_close").
    pub ffi_name: String,

    /// Receiver type: "ref", "mut", or "none" (for free functions).
    pub receiver: String,

    /// Input parameters (excluding self).
    pub params: Vec<SdkParam>,

    /// Rust return type as a string.
    pub return_type: SdkReturnType,

    /// FFI return type as a string.
    pub ffi_return_type: String,

    /// Out-parameters added to the FFI signature.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ffi_out_params: Vec<SdkParam>,
}

/// A parameter in the SDK manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkParam {
    /// Parameter name.
    pub name: String,

    /// Rust type as a string.
    #[serde(rename = "type")]
    pub param_type: String,

    /// If the Rust type is flattened (e.g., Vec2 -> x, y), these are the
    /// individual FFI parameters it expands to.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ffi_params: Vec<SdkParam>,
}

/// Return type representation in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdkReturnType {
    /// Simple type name (e.g., "bool", "f32").
    Simple(String),

    /// Result wrapper (e.g., {"result": "f32"}).
    Result { result: String },

    /// Option wrapper (e.g., {"option": "Contact"}).
    Option { option: String },
}

impl SdkReturnType {
    /// Creates a simple return type.
    pub fn simple(name: impl Into<String>) -> Self {
        Self::Simple(name.into())
    }

    /// Creates a Result return type.
    pub fn result(inner: impl Into<String>) -> Self {
        Self::Result {
            result: inner.into(),
        }
    }

    /// Creates an Option return type.
    pub fn option(inner: impl Into<String>) -> Self {
        Self::Option {
            option: inner.into(),
        }
    }
}

/// Generates a JSON string for the manifest metadata of a single module.
///
/// This is called during macro expansion to produce a const string that
/// build.rs can later collect.
pub fn generate_manifest_json(module: &SdkModule) -> String {
    serde_json::to_string_pretty(module)
        .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize manifest: {}\"}}", e))
}

/// Generates a const declaration that embeds manifest JSON.
///
/// The const is placed in a hidden module so it doesn't pollute the
/// namespace. Build.rs or a codegen script can find these by scanning
/// for the `GOUD_API_MANIFEST_` prefix.
pub fn manifest_const_name(module_name: &str) -> String {
    format!(
        "GOUD_API_MANIFEST_{}",
        module_name.to_uppercase().replace('-', "_")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_method_serialization() {
        let method = SdkMethod {
            name: "should_close".to_string(),
            ffi_name: "goud_window_should_close".to_string(),
            receiver: "ref".to_string(),
            params: vec![],
            return_type: SdkReturnType::simple("bool"),
            ffi_return_type: "bool".to_string(),
            ffi_out_params: vec![],
        };

        let json = serde_json::to_string_pretty(&method).unwrap();
        assert!(json.contains("should_close"));
        assert!(json.contains("goud_window_should_close"));
    }

    #[test]
    fn test_sdk_module_serialization() {
        let module = SdkModule {
            name: "window".to_string(),
            feature: Some("native".to_string()),
            methods: vec![SdkMethod {
                name: "should_close".to_string(),
                ffi_name: "goud_window_should_close".to_string(),
                receiver: "ref".to_string(),
                params: vec![],
                return_type: SdkReturnType::simple("bool"),
                ffi_return_type: "bool".to_string(),
                ffi_out_params: vec![],
            }],
            functions: vec![],
        };

        let json = generate_manifest_json(&module);
        assert!(json.contains("window"));
        assert!(json.contains("should_close"));
    }

    #[test]
    fn test_result_return_type() {
        let rt = SdkReturnType::result("f32");
        let json = serde_json::to_string(&rt).unwrap();
        assert!(json.contains("\"result\":\"f32\""));
    }

    #[test]
    fn test_option_return_type() {
        let rt = SdkReturnType::option("Contact");
        let json = serde_json::to_string(&rt).unwrap();
        assert!(json.contains("\"option\":\"Contact\""));
    }

    #[test]
    fn test_manifest_const_name() {
        assert_eq!(manifest_const_name("window"), "GOUD_API_MANIFEST_WINDOW");
        assert_eq!(
            manifest_const_name("renderer-3d"),
            "GOUD_API_MANIFEST_RENDERER_3D"
        );
    }
}
