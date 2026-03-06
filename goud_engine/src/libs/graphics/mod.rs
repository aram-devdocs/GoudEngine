//! Low-level graphics abstractions.
//!
//! This module contains the rendering backend trait, concrete backend
//! implementations (like OpenGL), and GPU resource management. The
//! `SpriteBatch` renderer lives in [`crate::rendering::sprite_batch`].

pub mod backend;
#[cfg(feature = "native")]
pub mod renderer3d;
