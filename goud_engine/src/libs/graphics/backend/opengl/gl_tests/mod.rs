//! Tests for the OpenGL backend, organized by domain.
//!
//! Tests marked `#[ignore]` require a live OpenGL context and are run
//! selectively in environments with display support.

#[cfg(test)]
mod buffer_tests;
#[cfg(test)]
mod conversion_tests;
#[cfg(test)]
mod draw_call_tests;
#[cfg(test)]
mod shader_tests;
#[cfg(test)]
mod texture_tests;
