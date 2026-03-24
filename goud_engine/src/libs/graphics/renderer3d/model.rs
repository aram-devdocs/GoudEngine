//! Model and model instance types for the 3D renderer.

use crate::assets::loaders::animation::KeyframeAnimation;
use crate::assets::loaders::mesh::{MeshBounds, SkeletonData};

/// A loaded 3D model consisting of one or more sub-mesh objects and materials.
///
/// A `Model3D` owns its GPU resources (buffers and materials) and is
/// identified by a model handle returned from [`Renderer3D::load_model`].
#[derive(Debug)]
#[allow(dead_code)] // Fields used in Phase B (animation, skeleton).
pub(in crate::libs::graphics::renderer3d) struct Model3D {
    /// Object IDs for each sub-mesh in the model.
    pub mesh_object_ids: Vec<u32>,
    /// Material IDs created for each sub-mesh.
    pub mesh_material_ids: Vec<u32>,
    /// AABB bounds of the full model.
    pub bounds: MeshBounds,
    /// Source file path this model was loaded from.
    pub source_path: String,
    /// Skeleton data for skinned animation (populated in Phase B).
    pub skeleton: Option<SkeletonData>,
    /// Animations embedded in the model file.
    pub animations: Vec<KeyframeAnimation>,
}

/// An instantiated copy of a [`Model3D`] with its own GPU resources.
///
/// Instances share the same logical source model but have independent
/// Object3D and Material3D entries so they can be positioned and
/// shaded independently.
#[derive(Debug)]
pub(in crate::libs::graphics::renderer3d) struct ModelInstance3D {
    /// The source model this instance was created from.
    pub source_model_id: u32,
    /// This instance's own Object3D IDs (separate from source).
    pub mesh_object_ids: Vec<u32>,
    /// This instance's material IDs.
    pub mesh_material_ids: Vec<u32>,
}
