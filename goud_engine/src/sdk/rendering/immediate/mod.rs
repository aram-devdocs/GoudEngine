mod draw;
mod math;
mod state;

pub use state::ImmediateRenderState;

pub(crate) use math::{model_matrix, ortho_matrix};
pub(crate) use state::{configure_immediate_vertex_layout, create_immediate_render_state};
