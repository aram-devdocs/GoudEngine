//! Tests for the 2D skeletal animation system.

use super::interpolation::sample_track;
use super::system::{deform_skeletal_meshes, update_skeletal_animations};
use crate::core::math::Vec2;
use crate::ecs::components::skeleton2d::{
    Bone2D, BoneKeyframe, BoneTrack, BoneTransform, BoneWeight, Skeleton2D, SkeletalAnimation,
    SkeletalAnimator, SkeletalMesh2D, SkeletalVertex,
};
use crate::ecs::World;
use std::f32::consts::FRAC_PI_2;

// =========================================================================
// Helper factories
// =========================================================================

/// Creates a simple two-bone skeleton: root at origin, child offset by (10,0).
fn two_bone_skeleton() -> Skeleton2D {
    let bones = vec![
        Bone2D {
            id: 0,
            name: "root".into(),
            parent_id: None,
            local_transform: BoneTransform::default(),
            bind_pose_inverse: BoneTransform::default(),
        },
        Bone2D {
            id: 1,
            name: "child".into(),
            parent_id: Some(0),
            local_transform: BoneTransform {
                position: Vec2::new(10.0, 0.0),
                ..BoneTransform::default()
            },
            bind_pose_inverse: BoneTransform {
                position: Vec2::new(-10.0, 0.0),
                ..BoneTransform::default()
            },
        },
    ];
    Skeleton2D::new(bones)
}

/// Creates an animation that rotates the root bone from 0 to PI/2 over 1 second.
fn root_rotation_animation() -> SkeletalAnimation {
    SkeletalAnimation {
        name: "rotate_root".into(),
        duration: 1.0,
        looping: false,
        tracks: vec![BoneTrack {
            bone_id: 0,
            keyframes: vec![
                BoneKeyframe {
                    time: 0.0,
                    transform: BoneTransform::default(),
                },
                BoneKeyframe {
                    time: 1.0,
                    transform: BoneTransform {
                        rotation: FRAC_PI_2,
                        ..BoneTransform::default()
                    },
                },
            ],
        }],
    }
}

// =========================================================================
// BoneTransform unit tests
// =========================================================================

#[test]
fn test_bone_transform_default_is_identity() {
    let t = BoneTransform::default();
    assert_eq!(t.position, Vec2::zero());
    assert_eq!(t.rotation, 0.0);
    assert_eq!(t.scale, Vec2::one());
}

#[test]
fn test_bone_transform_lerp_midpoint() {
    let a = BoneTransform::default();
    let b = BoneTransform {
        position: Vec2::new(10.0, 20.0),
        rotation: 2.0,
        scale: Vec2::new(3.0, 3.0),
    };
    let mid = a.lerp(b, 0.5);
    assert!((mid.position.x - 5.0).abs() < 1e-5);
    assert!((mid.position.y - 10.0).abs() < 1e-5);
    assert!((mid.rotation - 1.0).abs() < 1e-5);
    assert!((mid.scale.x - 2.0).abs() < 1e-5);
}

#[test]
fn test_bone_transform_to_matrix_identity() {
    let m = BoneTransform::default().to_matrix();
    assert!((m[0][0] - 1.0).abs() < 1e-5);
    assert!((m[1][1] - 1.0).abs() < 1e-5);
    assert!((m[2][2] - 1.0).abs() < 1e-5);
    assert!(m[0][1].abs() < 1e-5);
    assert!(m[1][0].abs() < 1e-5);
}

#[test]
fn test_bone_transform_combine_translation() {
    let parent = BoneTransform {
        position: Vec2::new(5.0, 3.0),
        ..BoneTransform::default()
    };
    let child = BoneTransform {
        position: Vec2::new(2.0, 1.0),
        ..BoneTransform::default()
    };
    let combined = BoneTransform::combine(&parent, &child);
    assert!((combined.position.x - 7.0).abs() < 1e-5);
    assert!((combined.position.y - 4.0).abs() < 1e-5);
}

// =========================================================================
// Hierarchy propagation tests
// =========================================================================

#[test]
fn test_bone_hierarchy_propagation() {
    let skel = two_bone_skeleton();
    // Root is at origin, child has local offset (10, 0).
    assert!((skel.world_transforms[0].position.x).abs() < 1e-5);
    assert!((skel.world_transforms[1].position.x - 10.0).abs() < 1e-5);
    assert!((skel.world_transforms[1].position.y).abs() < 1e-5);
}

#[test]
fn test_bone_hierarchy_propagation_with_parent_rotation() {
    let bones = vec![
        Bone2D {
            id: 0,
            name: "root".into(),
            parent_id: None,
            local_transform: BoneTransform {
                rotation: FRAC_PI_2,
                ..BoneTransform::default()
            },
            bind_pose_inverse: BoneTransform::default(),
        },
        Bone2D {
            id: 1,
            name: "child".into(),
            parent_id: Some(0),
            local_transform: BoneTransform {
                position: Vec2::new(10.0, 0.0),
                ..BoneTransform::default()
            },
            bind_pose_inverse: BoneTransform::default(),
        },
    ];
    let skel = Skeleton2D::new(bones);
    // Root rotated 90 degrees, child offset (10,0) -> world (0, 10)
    assert!((skel.world_transforms[1].position.x).abs() < 1e-4);
    assert!((skel.world_transforms[1].position.y - 10.0).abs() < 1e-4);
}

// =========================================================================
// Keyframe interpolation tests
// =========================================================================

#[test]
fn test_keyframe_interpolation_midpoint() {
    let keyframes = vec![
        BoneKeyframe {
            time: 0.0,
            transform: BoneTransform::default(),
        },
        BoneKeyframe {
            time: 1.0,
            transform: BoneTransform {
                position: Vec2::new(10.0, 0.0),
                ..BoneTransform::default()
            },
        },
    ];
    let sampled = sample_track(&keyframes, 0.5);
    assert!((sampled.position.x - 5.0).abs() < 1e-5);
}

#[test]
fn test_keyframe_interpolation_before_first() {
    let keyframes = vec![BoneKeyframe {
        time: 0.5,
        transform: BoneTransform {
            position: Vec2::new(10.0, 0.0),
            ..BoneTransform::default()
        },
    }];
    let sampled = sample_track(&keyframes, 0.0);
    assert!((sampled.position.x - 10.0).abs() < 1e-5);
}

#[test]
fn test_keyframe_interpolation_after_last() {
    let keyframes = vec![BoneKeyframe {
        time: 0.0,
        transform: BoneTransform {
            position: Vec2::new(7.0, 0.0),
            ..BoneTransform::default()
        },
    }];
    let sampled = sample_track(&keyframes, 99.0);
    assert!((sampled.position.x - 7.0).abs() < 1e-5);
}

#[test]
fn test_keyframe_interpolation_empty_returns_default() {
    let sampled = sample_track(&[], 0.5);
    assert_eq!(sampled, BoneTransform::default());
}

// =========================================================================
// Animation looping tests
// =========================================================================

#[test]
fn test_animation_looping_wraps_time() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, two_bone_skeleton());

    let mut anim = root_rotation_animation();
    anim.looping = true;

    let mut animator = SkeletalAnimator::new(anim);
    animator.play();
    world.insert(entity, animator);

    // Advance 1.5s on a 1.0s looping animation -> wraps to 0.5s
    update_skeletal_animations(&mut world, 1.5);

    let anim = world.get::<SkeletalAnimator>(entity).unwrap();
    assert!((anim.current_time - 0.5).abs() < 1e-5);
}

#[test]
fn test_animation_non_looping_clamps() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, two_bone_skeleton());

    let mut animator = SkeletalAnimator::new(root_rotation_animation());
    animator.play();
    world.insert(entity, animator);

    // Advance well past the 1.0s duration
    update_skeletal_animations(&mut world, 5.0);

    let anim = world.get::<SkeletalAnimator>(entity).unwrap();
    assert!((anim.current_time - 1.0).abs() < 1e-5);
    assert!(anim.is_finished());
}

// =========================================================================
// Mesh deformation tests
// =========================================================================

#[test]
fn test_mesh_deformation_single_bone_weight() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    // Single bone at origin with identity bind pose inverse.
    let bones = vec![Bone2D {
        id: 0,
        name: "root".into(),
        parent_id: None,
        local_transform: BoneTransform {
            position: Vec2::new(5.0, 0.0),
            ..BoneTransform::default()
        },
        bind_pose_inverse: BoneTransform::default(),
    }];
    world.insert(entity, Skeleton2D::new(bones));

    let mesh = SkeletalMesh2D::new(
        vec![SkeletalVertex {
            position: Vec2::new(1.0, 0.0),
            uv: Vec2::zero(),
            weights: vec![BoneWeight {
                bone_id: 0,
                weight: 1.0,
            }],
        }],
        vec![],
    );
    world.insert(entity, mesh);

    deform_skeletal_meshes(&mut world);

    let mesh = world.get::<SkeletalMesh2D>(entity).unwrap();
    // Bone at (5,0), vertex at rest (1,0) -> deformed (6,0)
    assert!((mesh.deformed_positions[0].x - 6.0).abs() < 1e-4);
    assert!((mesh.deformed_positions[0].y).abs() < 1e-4);
}

#[test]
fn test_mesh_deformation_multi_bone_weights() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    let bones = vec![
        Bone2D {
            id: 0,
            name: "a".into(),
            parent_id: None,
            local_transform: BoneTransform {
                position: Vec2::new(10.0, 0.0),
                ..BoneTransform::default()
            },
            bind_pose_inverse: BoneTransform::default(),
        },
        Bone2D {
            id: 1,
            name: "b".into(),
            parent_id: None,
            local_transform: BoneTransform {
                position: Vec2::new(0.0, 10.0),
                ..BoneTransform::default()
            },
            bind_pose_inverse: BoneTransform::default(),
        },
    ];
    world.insert(entity, Skeleton2D::new(bones));

    let mesh = SkeletalMesh2D::new(
        vec![SkeletalVertex {
            position: Vec2::zero(),
            uv: Vec2::zero(),
            weights: vec![
                BoneWeight {
                    bone_id: 0,
                    weight: 0.5,
                },
                BoneWeight {
                    bone_id: 1,
                    weight: 0.5,
                },
            ],
        }],
        vec![],
    );
    world.insert(entity, mesh);

    deform_skeletal_meshes(&mut world);

    let mesh = world.get::<SkeletalMesh2D>(entity).unwrap();
    // 50% bone_a (10,0) + 50% bone_b (0,10) on rest pos (0,0) -> (5, 5)
    assert!((mesh.deformed_positions[0].x - 5.0).abs() < 1e-4);
    assert!((mesh.deformed_positions[0].y - 5.0).abs() < 1e-4);
}

// =========================================================================
// Integration test: two-bone animation playback
// =========================================================================

#[test]
fn test_two_bone_animation_playback() {
    let mut world = World::new();
    let entity = world.spawn_empty();

    // Two-bone skeleton: root at origin, child at (10, 0).
    world.insert(entity, two_bone_skeleton());

    // Animate the root bone rotating from 0 to PI/2 over 1 second.
    let mut animator = SkeletalAnimator::new(root_rotation_animation());
    animator.play();
    world.insert(entity, animator);

    // Advance to t=0.5 -> root rotation should be ~PI/4 (45 degrees).
    update_skeletal_animations(&mut world, 0.5);

    let skel = world.get::<Skeleton2D>(entity).unwrap();
    let expected_angle = FRAC_PI_2 * 0.5; // PI/4

    // Root bone world rotation should be ~PI/4.
    assert!(
        (skel.world_transforms[0].rotation - expected_angle).abs() < 1e-4,
        "Root rotation at t=0.5 should be ~PI/4, got {}",
        skel.world_transforms[0].rotation
    );

    // Child bone: local offset (10, 0) rotated by PI/4 around origin.
    // Expected world position: (10*cos(PI/4), 10*sin(PI/4)) ~ (7.07, 7.07)
    let child_pos = skel.world_transforms[1].position;
    let expected_x = 10.0 * expected_angle.cos();
    let expected_y = 10.0 * expected_angle.sin();
    assert!(
        (child_pos.x - expected_x).abs() < 1e-3,
        "Child x should be ~{}, got {}",
        expected_x,
        child_pos.x
    );
    assert!(
        (child_pos.y - expected_y).abs() < 1e-3,
        "Child y should be ~{}, got {}",
        expected_y,
        child_pos.y
    );
}

// =========================================================================
// Skeleton utility tests
// =========================================================================

#[test]
fn test_skeleton_find_bone() {
    let skel = two_bone_skeleton();
    assert_eq!(skel.find_bone("root"), Some(0));
    assert_eq!(skel.find_bone("child"), Some(1));
    assert_eq!(skel.find_bone("nonexistent"), None);
}

#[test]
fn test_skeleton_bone_count() {
    let skel = two_bone_skeleton();
    assert_eq!(skel.bone_count(), 2);
}

// =========================================================================
// Animator playback control tests
// =========================================================================

#[test]
fn test_animator_play_pause_reset() {
    let mut animator = SkeletalAnimator::new(root_rotation_animation());
    assert!(!animator.playing);

    animator.play();
    assert!(animator.playing);

    animator.pause();
    assert!(!animator.playing);

    animator.current_time = 0.5;
    animator.reset();
    assert_eq!(animator.current_time, 0.0);
}

#[test]
fn test_animator_is_finished() {
    let mut animator = SkeletalAnimator::new(root_rotation_animation());
    assert!(!animator.is_finished());

    animator.current_time = 1.0;
    assert!(animator.is_finished());

    // Looping animation is never finished
    animator.animation.looping = true;
    assert!(!animator.is_finished());
}

#[test]
fn test_paused_animator_does_not_advance() {
    let mut world = World::new();
    let entity = world.spawn_empty();
    world.insert(entity, two_bone_skeleton());

    let animator = SkeletalAnimator::new(root_rotation_animation()); // starts paused
    world.insert(entity, animator);

    update_skeletal_animations(&mut world, 0.5);

    let anim = world.get::<SkeletalAnimator>(entity).unwrap();
    assert_eq!(anim.current_time, 0.0);
}
