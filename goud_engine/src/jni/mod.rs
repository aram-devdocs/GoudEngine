//! Internal JNI bridge used by the JVM-facing bindings.
//
// Keep this module name even though it matches the external `jni` crate name.
// Inside generated code, `jni::` resolves to the dependency crate, while
// `crate::jni::` resolves to this bridge module.

mod helpers;

#[rustfmt::skip]
#[path = "generated.g.rs"]
mod generated;

#[cfg(test)]
mod tests;

pub use generated::*;
