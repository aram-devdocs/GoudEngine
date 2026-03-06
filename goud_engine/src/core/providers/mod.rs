//! Provider registry and configuration for GoudEngine.
//!
//! This module contains the [`ProviderRegistry`] which holds all engine
//! providers, and the [`ProviderRegistryBuilder`] for constructing a
//! registry with custom or default (null) providers.

mod builder;
mod registry;

pub use builder::ProviderRegistryBuilder;
pub use registry::ProviderRegistry;
