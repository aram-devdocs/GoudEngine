//! Low-level graphics abstractions and systems.
//!
//! This module contains the rendering backend trait, concrete backend
//! implementations (like OpenGL), and GPU resource management. It also
//! includes higher-level systems like the `SpriteBatch` renderer and
//! the complete 3D rendering system.

pub mod backend;
pub mod renderer3d;
pub mod sprite_batch;
