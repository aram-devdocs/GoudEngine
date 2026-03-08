//! Fallback asset registry for substituting defaults on load failure.
//!
//! When an asset fails to load, the [`FallbackRegistry`] provides a clone of
//! a pre-registered default asset so the handle still points to usable data.

mod registry;

#[cfg(test)]
mod tests;

pub use registry::FallbackRegistry;
