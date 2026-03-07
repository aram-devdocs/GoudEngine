//! Core skeletal animation system functions.
//!
//! Two system functions drive the skeletal animation pipeline each frame:
//!
//! 1. [`update_skeletal_animations`] -- advances time, samples keyframes,
//!    propagates bone hierarchy.
//! 2. [`deform_skeletal_meshes`] -- applies bone transforms to mesh vertices
//!    via linear blend skinning.

use super::interpolation::sample_track;
use crate::core::math::Vec2;
use crate::ecs::components::skeleton2d::{
    BoneTransform, SkeletalAnimator, SkeletalMesh2D, Skeleton2D,
};
use crate::ecs::World;

/// Advances skeletal animations and propagates bone transforms.
///
/// For each entity with both [`SkeletalAnimator`] and [`Skeleton2D`]:
///
/// 1. Advance `current_time` by `dt * speed`.
/// 2. Handle looping (wrap) or clamping for non-looping animations.
/// 3. Sample each bone track at the current time to get local transforms.
/// 4. Propagate through the bone hierarchy to compute world transforms.
pub fn update_skeletal_animations(world: &mut World, dt: f32) {
    let entities: Vec<_> = world
        .archetypes()
        .iter()
        .flat_map(|archetype| archetype.entities().iter().copied())
        .filter(|&entity| world.has::<SkeletalAnimator>(entity) && world.has::<Skeleton2D>(entity))
        .collect();

    for entity in entities {
        // --- Phase 1: advance time on the animator ---
        let (tracks, current_time, bone_count) = {
            let Some(animator) = world.get_mut::<SkeletalAnimator>(entity) else {
                continue;
            };

            if !animator.playing || animator.animation.duration <= 0.0 {
                continue;
            }

            animator.current_time += dt * animator.speed;

            if animator.animation.looping {
                while animator.current_time >= animator.animation.duration {
                    animator.current_time -= animator.animation.duration;
                }
                while animator.current_time < 0.0 {
                    animator.current_time += animator.animation.duration;
                }
            } else {
                animator.current_time = animator
                    .current_time
                    .clamp(0.0, animator.animation.duration);
            }

            let time = animator.current_time;
            let tracks = animator.animation.tracks.clone();
            let bone_count = {
                let Some(skel) = world.get::<Skeleton2D>(entity) else {
                    continue;
                };
                skel.bone_count()
            };
            (tracks, time, bone_count)
        };

        // --- Phase 2: sample keyframes into local transforms ---
        let mut local_transforms = vec![BoneTransform::default(); bone_count];

        // Start from the skeleton's stored local transforms as defaults.
        {
            let Some(skel) = world.get::<Skeleton2D>(entity) else {
                continue;
            };
            for (i, bone) in skel.bones.iter().enumerate() {
                local_transforms[i] = bone.local_transform;
            }
        }

        // Override with sampled keyframe data for animated bones.
        for track in &tracks {
            if track.bone_id < bone_count {
                local_transforms[track.bone_id] = sample_track(&track.keyframes, current_time);
            }
        }

        // --- Phase 3: propagate hierarchy to world transforms ---
        // Collect parent IDs so we can do indexed parent lookups on world_transforms.
        let parent_ids: Vec<Option<usize>> = {
            let Some(skel) = world.get::<Skeleton2D>(entity) else {
                continue;
            };
            skel.bones.iter().map(|b| b.parent_id).collect()
        };

        let Some(skel) = world.get_mut::<Skeleton2D>(entity) else {
            continue;
        };

        for (i, (local, parent_id)) in local_transforms.iter().zip(parent_ids.iter()).enumerate() {
            skel.world_transforms[i] = match parent_id {
                Some(pid) => BoneTransform::combine(&skel.world_transforms[*pid], local),
                None => *local,
            };
        }
    }
}

/// Deforms skeletal meshes using linear blend skinning.
///
/// For each entity with both [`Skeleton2D`] and [`SkeletalMesh2D`]:
///
/// 1. For each vertex, compute a weighted sum of
///    `bone_world_transform * bind_pose_inverse * rest_position`.
/// 2. Store the result in [`SkeletalMesh2D::deformed_positions`].
pub fn deform_skeletal_meshes(world: &mut World) {
    let entities: Vec<_> = world
        .archetypes()
        .iter()
        .flat_map(|archetype| archetype.entities().iter().copied())
        .filter(|&entity| world.has::<Skeleton2D>(entity) && world.has::<SkeletalMesh2D>(entity))
        .collect();

    for entity in entities {
        // Gather bone data (world transforms + bind pose inverses).
        let bone_data: Vec<(BoneTransform, BoneTransform)> = {
            let Some(skel) = world.get::<Skeleton2D>(entity) else {
                continue;
            };
            skel.bones
                .iter()
                .zip(skel.world_transforms.iter())
                .map(|(bone, world_t)| (*world_t, bone.bind_pose_inverse))
                .collect()
        };

        let Some(mesh) = world.get_mut::<SkeletalMesh2D>(entity) else {
            continue;
        };

        for (vi, vertex) in mesh.vertices.iter().enumerate() {
            let rest = vertex.position;
            let mut result = Vec2::zero();

            for bw in &vertex.weights {
                if bw.bone_id >= bone_data.len() {
                    continue;
                }
                let (world_t, bind_inv) = &bone_data[bw.bone_id];
                let skinning = BoneTransform::combine(world_t, bind_inv);
                let deformed = transform_point(&skinning, rest);
                result = result + deformed * bw.weight;
            }

            mesh.deformed_positions[vi] = result;
        }
    }
}

/// Applies a [`BoneTransform`] to a 2D point (scale, rotate, translate).
fn transform_point(t: &BoneTransform, p: Vec2) -> Vec2 {
    let scaled = Vec2::new(p.x * t.scale.x, p.y * t.scale.y);
    let (sin, cos) = t.rotation.sin_cos();
    let rotated = Vec2::new(
        scaled.x * cos - scaled.y * sin,
        scaled.x * sin + scaled.y * cos,
    );
    Vec2::new(t.position.x + rotated.x, t.position.y + rotated.y)
}
