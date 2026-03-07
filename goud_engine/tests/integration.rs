//! Cross-layer integration tests spanning ECS, FFI, and component systems.
//!
//! These tests verify correct behavior across module boundaries without
//! requiring a GPU or display.

#[path = "integration/ecs_integration.rs"]
mod ecs_integration;
#[path = "integration/ffi_cross_layer.rs"]
mod ffi_cross_layer;
#[path = "integration/helpers.rs"]
mod helpers;
