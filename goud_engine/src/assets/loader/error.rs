//! Asset loading error types.

use std::error::Error;
use std::fmt;
use std::path::Path;

/// Errors that can occur during asset loading.
#[derive(Debug, Clone)]
pub enum AssetLoadError {
    /// The asset file was not found.
    NotFound {
        /// The path to the asset that was not found.
        path: String,
    },

    /// Failed to read the asset file.
    IoError {
        /// The path to the asset that failed to load.
        path: String,
        /// The I/O error message.
        message: String,
    },

    /// Failed to decode/parse the asset data.
    DecodeFailed(
        /// The decoding error message.
        String,
    ),

    /// The asset format is not supported.
    UnsupportedFormat {
        /// The unsupported file extension.
        extension: String,
    },

    /// A dependency asset failed to load.
    DependencyFailed {
        /// The path of the asset with the failed dependency.
        asset_path: String,
        /// The path of the dependency that failed to load.
        dependency_path: String,
        /// The error message from the dependency failure.
        message: String,
    },

    /// A custom loader-specific error occurred.
    Custom(
        /// The custom error message.
        String,
    ),
}

impl AssetLoadError {
    /// Creates a NotFound error from a path.
    pub fn not_found(path: impl AsRef<Path>) -> Self {
        Self::NotFound {
            path: path.as_ref().display().to_string(),
        }
    }

    /// Creates an IoError from a path and error.
    pub fn io_error(path: impl AsRef<Path>, error: impl Error) -> Self {
        Self::IoError {
            path: path.as_ref().display().to_string(),
            message: error.to_string(),
        }
    }

    /// Creates a DecodeFailed error from a message.
    pub fn decode_failed(message: impl Into<String>) -> Self {
        Self::DecodeFailed(message.into())
    }

    /// Creates an UnsupportedFormat error from an extension.
    pub fn unsupported_format(extension: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            extension: extension.into(),
        }
    }

    /// Creates a DependencyFailed error.
    pub fn dependency_failed(
        asset_path: impl Into<String>,
        dependency_path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::DependencyFailed {
            asset_path: asset_path.into(),
            dependency_path: dependency_path.into(),
            message: message.into(),
        }
    }

    /// Creates a Custom error from a message.
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    /// Returns true if this is a NotFound error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Returns true if this is an IoError.
    pub fn is_io_error(&self) -> bool {
        matches!(self, Self::IoError { .. })
    }

    /// Returns true if this is a DecodeFailed error.
    pub fn is_decode_failed(&self) -> bool {
        matches!(self, Self::DecodeFailed(_))
    }

    /// Returns true if this is an UnsupportedFormat error.
    pub fn is_unsupported_format(&self) -> bool {
        matches!(self, Self::UnsupportedFormat { .. })
    }

    /// Returns true if this is a DependencyFailed error.
    pub fn is_dependency_failed(&self) -> bool {
        matches!(self, Self::DependencyFailed { .. })
    }

    /// Returns true if this is a Custom error.
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }
}

impl fmt::Display for AssetLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { path } => write!(f, "Asset not found: {}", path),
            Self::IoError { path, message } => {
                write!(f, "I/O error loading asset '{}': {}", path, message)
            }
            Self::DecodeFailed(msg) => write!(f, "Failed to decode asset: {}", msg),
            Self::UnsupportedFormat { extension } => {
                write!(f, "Unsupported asset format: '.{}'", extension)
            }
            Self::DependencyFailed {
                asset_path,
                dependency_path,
                message,
            } => write!(
                f,
                "Dependency '{}' of asset '{}' failed to load: {}",
                dependency_path, asset_path, message
            ),
            Self::Custom(msg) => write!(f, "Asset loading error: {}", msg),
        }
    }
}

impl Error for AssetLoadError {}
