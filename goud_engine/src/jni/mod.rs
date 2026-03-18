//! Internal JNI bridge used by the JVM-facing bindings.

mod helpers;

#[rustfmt::skip]
mod generated;

#[cfg(test)]
mod tests;

pub use generated::*;
