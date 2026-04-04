//! Bone sampling, channel maps, and matrix computation for skeletal animation.
//!
//! Extracted from `animation.rs` to keep files under 500 lines.

use crate::core::types::{interpolate, KeyframeAnimation, SkeletonData};

use super::animation::IDENTITY_MAT4;

/// Pre-computed property name strings for a single bone (10 channels).
///
/// Built once per skeleton at model load time and cached on [`Model3D`] so
/// that `compute_bone_matrices` never allocates per-frame `format!()` strings.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct BonePropertyNames {
    pub(in crate::libs::graphics::renderer3d) translation: [String; 3],
    pub(in crate::libs::graphics::renderer3d) rotation: [String; 4],
    pub(in crate::libs::graphics::renderer3d) scale: [String; 3],
}

impl BonePropertyNames {
    pub fn new(bone_idx: usize) -> Self {
        Self {
            translation: [
                format!("node_{bone_idx}.translation.x"),
                format!("node_{bone_idx}.translation.y"),
                format!("node_{bone_idx}.translation.z"),
            ],
            rotation: [
                format!("node_{bone_idx}.rotation.x"),
                format!("node_{bone_idx}.rotation.y"),
                format!("node_{bone_idx}.rotation.z"),
                format!("node_{bone_idx}.rotation.w"),
            ],
            scale: [
                format!("node_{bone_idx}.scale.x"),
                format!("node_{bone_idx}.scale.y"),
                format!("node_{bone_idx}.scale.z"),
            ],
        }
    }
}

/// Pre-computed channel index map for a (skeleton, animation) pair.
///
/// Eliminates per-frame string HashMap lookups by mapping each bone directly
/// to the indices of its 10 animation channels (tx, ty, tz, rx, ry, rz, rw,
/// sx, sy, sz). Built once at model load time.
#[derive(Debug, Clone)]
pub struct BoneChannelMap {
    /// For each bone: `[tx, ty, tz, rx, ry, rz, rw, sx, sy, sz]` channel indices.
    /// `None` means that channel does not exist in this animation.
    pub channels: Vec<[Option<usize>; 10]>,
}

impl BoneChannelMap {
    /// Build a channel map for the given skeleton and animation.
    pub fn build(skeleton: &SkeletonData, anim: &KeyframeAnimation) -> Self {
        let bone_count = skeleton.bones.len();
        let mut channels = Vec::with_capacity(bone_count);
        for bone_idx in 0..bone_count {
            let find = |suffix: &str| -> Option<usize> {
                let prop = format!("node_{bone_idx}.{suffix}");
                anim.channels
                    .iter()
                    .position(|ch| ch.target_property == prop)
            };
            channels.push([
                find("translation.x"),
                find("translation.y"),
                find("translation.z"),
                find("rotation.x"),
                find("rotation.y"),
                find("rotation.z"),
                find("rotation.w"),
                find("scale.x"),
                find("scale.y"),
                find("scale.z"),
            ]);
        }
        Self { channels }
    }
}

/// Compute bone matrices using pre-cached property names (zero per-frame allocation).
#[inline]
pub(in crate::libs::graphics::renderer3d) fn compute_bone_matrices_with_names(
    skeleton: &SkeletonData,
    anim: &KeyframeAnimation,
    time: f32,
    prop_names: &[BonePropertyNames],
) -> Vec<[f32; 16]> {
    let bone_count = skeleton.bones.len();
    let mut local_transforms = Vec::with_capacity(bone_count);
    let mut global_transforms = vec![IDENTITY_MAT4; bone_count];
    let mut result = vec![IDENTITY_MAT4; bone_count];

    for names in prop_names.iter().take(bone_count) {
        let (tx, ty, tz) = sample_translation_indexed(anim, names, time);
        let (rx, ry, rz, rw) = sample_rotation_indexed(anim, names, time);
        let (sx, sy, sz) = sample_scale_indexed(anim, names, time);

        let local = build_trs_matrix(tx, ty, tz, rx, ry, rz, rw, sx, sy, sz);
        local_transforms.push(local);
    }

    walk_hierarchy(skeleton, &local_transforms, &mut global_transforms);

    for i in 0..bone_count {
        result[i] = mat4_mul(
            &global_transforms[i],
            &skeleton.bones[i].inverse_bind_matrix,
        );
    }

    result
}

/// Compute bone matrices into pre-allocated scratch buffers (zero per-frame allocation).
#[inline]
pub(in crate::libs::graphics::renderer3d) fn compute_bone_matrices_into(
    skeleton: &SkeletonData,
    anim: &KeyframeAnimation,
    time: f32,
    prop_names: &[BonePropertyNames],
    local_transforms: &mut [[f32; 16]],
    global_transforms: &mut [[f32; 16]],
    result: &mut [[f32; 16]],
) {
    let bone_count = skeleton.bones.len();

    for (i, names) in prop_names.iter().enumerate().take(bone_count) {
        let (tx, ty, tz) = sample_translation_indexed(anim, names, time);
        let (rx, ry, rz, rw) = sample_rotation_indexed(anim, names, time);
        let (sx, sy, sz) = sample_scale_indexed(anim, names, time);

        local_transforms[i] = build_trs_matrix(tx, ty, tz, rx, ry, rz, rw, sx, sy, sz);
    }

    walk_hierarchy(skeleton, local_transforms, global_transforms);

    for i in 0..bone_count {
        result[i] = mat4_mul(
            &global_transforms[i],
            &skeleton.bones[i].inverse_bind_matrix,
        );
    }
}

/// Compute bone matrices using pre-built [`BoneChannelMap`] (zero string lookups).
#[inline]
pub(in crate::libs::graphics::renderer3d) fn compute_bone_matrices_into_fast(
    skeleton: &SkeletonData,
    anim: &KeyframeAnimation,
    time: f32,
    channel_map: &BoneChannelMap,
    local_transforms: &mut [[f32; 16]],
    global_transforms: &mut [[f32; 16]],
    result: &mut [[f32; 16]],
) {
    let bone_count = skeleton.bones.len();

    for (cm, local_out) in channel_map
        .channels
        .iter()
        .zip(local_transforms.iter_mut())
        .take(bone_count)
    {
        let sample = |idx: Option<usize>, default: f32| -> f32 {
            match idx {
                Some(ch_idx) => interpolate(&anim.channels[ch_idx].keyframes, time),
                None => default,
            }
        };

        let tx = sample(cm[0], 0.0);
        let ty = sample(cm[1], 0.0);
        let tz = sample(cm[2], 0.0);
        let rx = sample(cm[3], 0.0);
        let ry = sample(cm[4], 0.0);
        let rz = sample(cm[5], 0.0);
        let rw = sample(cm[6], 1.0);
        let sx = sample(cm[7], 1.0);
        let sy = sample(cm[8], 1.0);
        let sz = sample(cm[9], 1.0);

        // Normalize quaternion (1 reciprocal + 4 multiplications instead of 4 divisions).
        let len = (rx * rx + ry * ry + rz * rz + rw * rw).sqrt();
        let (rx, ry, rz, rw) = if len > f32::EPSILON {
            let inv_len = 1.0 / len;
            (rx * inv_len, ry * inv_len, rz * inv_len, rw * inv_len)
        } else {
            (0.0, 0.0, 0.0, 1.0)
        };

        *local_out = build_trs_matrix(tx, ty, tz, rx, ry, rz, rw, sx, sy, sz);
    }

    walk_hierarchy(skeleton, local_transforms, global_transforms);

    for i in 0..bone_count {
        result[i] = mat4_mul(
            &global_transforms[i],
            &skeleton.bones[i].inverse_bind_matrix,
        );
    }
}

/// Compute bone matrices using channel maps (allocating path for blending).
pub(in crate::libs::graphics::renderer3d) fn compute_bone_matrices_with_channel_map(
    skeleton: &SkeletonData,
    anim: &KeyframeAnimation,
    time: f32,
    channel_map: &BoneChannelMap,
) -> Vec<[f32; 16]> {
    let bone_count = skeleton.bones.len();
    let mut local_transforms = vec![IDENTITY_MAT4; bone_count];
    let mut global_transforms = vec![IDENTITY_MAT4; bone_count];
    let mut result = vec![IDENTITY_MAT4; bone_count];

    compute_bone_matrices_into_fast(
        skeleton,
        anim,
        time,
        channel_map,
        &mut local_transforms,
        &mut global_transforms,
        &mut result,
    );

    result
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Walk the bone hierarchy to compute global transforms from local transforms.
#[inline]
fn walk_hierarchy(
    skeleton: &SkeletonData,
    local_transforms: &[[f32; 16]],
    global_transforms: &mut [[f32; 16]],
) {
    let bone_count = skeleton.bones.len();
    for i in 0..bone_count {
        let parent = skeleton.bones[i].parent_index;
        if parent >= 0 && (parent as usize) < bone_count {
            global_transforms[i] =
                mat4_mul(&global_transforms[parent as usize], &local_transforms[i]);
        } else {
            global_transforms[i] = local_transforms[i];
        }
    }
}

fn sample_translation_indexed(
    anim: &KeyframeAnimation,
    names: &BonePropertyNames,
    time: f32,
) -> (f32, f32, f32) {
    let x = sample_channel(anim, &names.translation[0], time).unwrap_or(0.0);
    let y = sample_channel(anim, &names.translation[1], time).unwrap_or(0.0);
    let z = sample_channel(anim, &names.translation[2], time).unwrap_or(0.0);
    (x, y, z)
}

fn sample_rotation_indexed(
    anim: &KeyframeAnimation,
    names: &BonePropertyNames,
    time: f32,
) -> (f32, f32, f32, f32) {
    let x = sample_channel(anim, &names.rotation[0], time).unwrap_or(0.0);
    let y = sample_channel(anim, &names.rotation[1], time).unwrap_or(0.0);
    let z = sample_channel(anim, &names.rotation[2], time).unwrap_or(0.0);
    let w = sample_channel(anim, &names.rotation[3], time).unwrap_or(1.0);
    // Normalize quaternion (1 reciprocal + 4 multiplications instead of 4 divisions).
    let len = (x * x + y * y + z * z + w * w).sqrt();
    if len > f32::EPSILON {
        let inv_len = 1.0 / len;
        (x * inv_len, y * inv_len, z * inv_len, w * inv_len)
    } else {
        (0.0, 0.0, 0.0, 1.0)
    }
}

fn sample_scale_indexed(
    anim: &KeyframeAnimation,
    names: &BonePropertyNames,
    time: f32,
) -> (f32, f32, f32) {
    let x = sample_channel(anim, &names.scale[0], time).unwrap_or(1.0);
    let y = sample_channel(anim, &names.scale[1], time).unwrap_or(1.0);
    let z = sample_channel(anim, &names.scale[2], time).unwrap_or(1.0);
    (x, y, z)
}

fn sample_channel(anim: &KeyframeAnimation, property: &str, time: f32) -> Option<f32> {
    anim.channel_by_property(property)
        .map(|ch| interpolate(&ch.keyframes, time))
}

/// Build a column-major 4x4 TRS matrix from translation, quaternion rotation, and scale.
#[inline]
pub(crate) fn build_trs_matrix(
    tx: f32,
    ty: f32,
    tz: f32,
    qx: f32,
    qy: f32,
    qz: f32,
    qw: f32,
    sx: f32,
    sy: f32,
    sz: f32,
) -> [f32; 16] {
    let xx = qx * qx;
    let yy = qy * qy;
    let zz = qz * qz;
    let xy = qx * qy;
    let xz = qx * qz;
    let yz = qy * qz;
    let wx = qw * qx;
    let wy = qw * qy;
    let wz = qw * qz;

    [
        sx * (1.0 - 2.0 * (yy + zz)),
        sx * (2.0 * (xy + wz)),
        sx * (2.0 * (xz - wy)),
        0.0,
        sy * (2.0 * (xy - wz)),
        sy * (1.0 - 2.0 * (xx + zz)),
        sy * (2.0 * (yz + wx)),
        0.0,
        sz * (2.0 * (xz + wy)),
        sz * (2.0 * (yz - wx)),
        sz * (1.0 - 2.0 * (xx + yy)),
        0.0,
        tx,
        ty,
        tz,
        1.0,
    ]
}

/// Multiply two column-major 4x4 matrices.
///
/// Unrolled inner loop to help the compiler auto-vectorize.
#[inline]
pub(crate) fn mat4_mul(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    // Column-major: out[col*4+row] = sum_k(a[k*4+row] * b[col*4+k])
    let mut out = [0.0f32; 16];
    for col in 0..4 {
        let b0 = b[col * 4];
        let b1 = b[col * 4 + 1];
        let b2 = b[col * 4 + 2];
        let b3 = b[col * 4 + 3];
        for row in 0..4 {
            out[col * 4 + row] = a[row] * b0 + a[4 + row] * b1 + a[8 + row] * b2 + a[12 + row] * b3;
        }
    }
    out
}
