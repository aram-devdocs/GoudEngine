//! FBX animation extraction: keyframes, Euler-to-quaternion conversion.

use std::collections::HashMap;

use crate::assets::loaders::animation::keyframe::{AnimationChannel, EasingFunction, Keyframe};
use crate::assets::loaders::animation::KeyframeAnimation;
use crate::assets::loaders::mesh::provider::SkeletonData;

use fbxcel::low::v7400::AttributeValue;
use fbxcel::tree::v7400::NodeHandle;

use super::helpers::{
    find_f32_array, find_f64_array, find_i64_array, strip_fbx_name, FbxConnections,
};

/// FBX time ticks per second (standard FBX time unit from FBX SDK).
const FBX_TICKS_PER_SECOND: f64 = 46_186_158_000.0;

struct CurveData {
    key_times: Vec<i64>,
    key_values: Vec<f32>,
}

/// Extracts keyframe animations from FBX AnimationStack/AnimationLayer nodes.
///
/// Walks the AnimationStack -> AnimationLayer -> AnimationCurveNode ->
/// AnimationCurve hierarchy, mapping each curve to a bone index via the
/// connection graph. Euler rotation curves are converted to quaternion
/// channels. Returns an empty `Vec` when no skeleton is present.
pub(super) fn extract_animations(
    root: &NodeHandle,
    conns: &FbxConnections,
    skeleton: &Option<SkeletonData>,
) -> Vec<KeyframeAnimation> {
    let skeleton = match skeleton {
        Some(s) => s,
        None => return Vec::new(),
    };

    // Build bone name -> bone index map for looking up bones by FBX name.
    let bone_name_to_idx: HashMap<&str, usize> = skeleton
        .bones
        .iter()
        .enumerate()
        .map(|(i, b)| (b.name.as_str(), i))
        .collect();

    // Collect all Model node IDs with their names (needed to resolve bone connections).
    let mut model_id_to_name: HashMap<i64, String> = HashMap::new();
    let mut model_name_to_id: HashMap<String, i64> = HashMap::new();
    for objects_node in root.children_by_name("Objects") {
        for model in objects_node.children_by_name("Model") {
            let attrs = model.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);
            let raw_name = attrs.get(1).and_then(|a| a.get_string()).unwrap_or("");
            let name = strip_fbx_name(raw_name);
            model_id_to_name.insert(obj_id, name.clone());
            model_name_to_id.insert(name, obj_id);
        }
    }

    // Build a map from Model ID -> bone index.
    let mut model_id_to_bone_idx: HashMap<i64, usize> = HashMap::new();
    for (name, &model_id) in &model_name_to_id {
        if let Some(&idx) = bone_name_to_idx.get(name.as_str()) {
            model_id_to_bone_idx.insert(model_id, idx);
        }
    }

    // Collect AnimationStack, AnimationLayer, AnimationCurveNode, AnimationCurve nodes.
    let mut anim_stacks: Vec<(i64, String)> = Vec::new();
    let mut anim_curve_nodes_map: HashMap<i64, NodeHandle> = HashMap::new();

    for objects_node in root.children_by_name("Objects") {
        for node in objects_node.children_by_name("AnimationStack") {
            let attrs = node.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);
            let raw_name = attrs.get(1).and_then(|a| a.get_string()).unwrap_or("");
            anim_stacks.push((obj_id, strip_fbx_name(raw_name)));
        }
        for node in objects_node.children_by_name("AnimationCurveNode") {
            let attrs = node.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);
            anim_curve_nodes_map.insert(obj_id, node);
        }
    }

    // Collect AnimationCurve nodes (KeyTime + KeyValueFloat).
    let mut anim_curves: HashMap<i64, CurveData> = HashMap::new();
    for objects_node in root.children_by_name("Objects") {
        for node in objects_node.children_by_name("AnimationCurve") {
            let attrs = node.attributes();
            let obj_id = attrs.first().and_then(AttributeValue::get_i64).unwrap_or(0);

            let key_times: Vec<i64> = find_i64_array(&node, "KeyTime")
                .map(|s| s.to_vec())
                .unwrap_or_default();
            let key_values: Vec<f32> = find_f32_array(&node, "KeyValueFloat")
                .map(|s| s.to_vec())
                .or_else(|| {
                    find_f64_array(&node, "KeyValueFloat")
                        .map(|s| s.iter().map(|&v| v as f32).collect())
                })
                .unwrap_or_default();

            if !key_times.is_empty() && key_times.len() == key_values.len() {
                anim_curves.insert(
                    obj_id,
                    CurveData {
                        key_times,
                        key_values,
                    },
                );
            }
        }
    }

    // For each AnimationStack, build a KeyframeAnimation.
    let mut animations: Vec<KeyframeAnimation> = Vec::new();

    for (stack_id, stack_name) in &anim_stacks {
        // Find layers connected to this stack.
        let layer_ids: Vec<i64> = conns.children_of.get(stack_id).cloned().unwrap_or_default();

        // Collect per-bone Euler rotation curves for later quaternion conversion.
        // bone_index -> (component_index, Vec<(time_sec, value_deg)>)
        let mut bone_euler_curves: HashMap<usize, [Vec<(f32, f32)>; 3]> = HashMap::new();
        let mut channels: Vec<AnimationChannel> = Vec::new();
        let mut max_time: f32 = 0.0;

        for &layer_id in &layer_ids {
            // Find CurveNodes connected to this layer.
            let curve_node_ids_for_layer: Vec<i64> = conns
                .children_of
                .get(&layer_id)
                .cloned()
                .unwrap_or_default();

            for &cn_id in &curve_node_ids_for_layer {
                if !anim_curve_nodes_map.contains_key(&cn_id) {
                    continue;
                }

                // Find which bone this CurveNode targets (via OP connection).
                let (bone_idx, property) =
                    match find_curve_node_target(cn_id, conns, &model_id_to_bone_idx) {
                        Some(v) => v,
                        None => continue,
                    };

                // Find the individual AnimationCurve children (X, Y, Z components).
                let child_curve_ids = conns.children_of.get(&cn_id).cloned().unwrap_or_default();

                for &curve_id in &child_curve_ids {
                    let curve = match anim_curves.get(&curve_id) {
                        Some(c) => c,
                        None => continue,
                    };

                    // Determine component from OP property: "d|X", "d|Y", "d|Z".
                    let component = conns
                        .properties
                        .get(&(curve_id, cn_id))
                        .map(|p| p.as_str())
                        .unwrap_or("");
                    let suffix = match component {
                        "d|X" => "x",
                        "d|Y" => "y",
                        "d|Z" => "z",
                        _ => continue,
                    };
                    let comp_idx = match suffix {
                        "x" => 0usize,
                        "y" => 1,
                        "z" => 2,
                        _ => continue,
                    };

                    // Convert FBX ticks to seconds and build keyframes.
                    let keyframes: Vec<(f32, f32)> = curve
                        .key_times
                        .iter()
                        .zip(curve.key_values.iter())
                        .map(|(&t, &v)| {
                            let time_sec = (t as f64 / FBX_TICKS_PER_SECOND) as f32;
                            if time_sec > max_time {
                                max_time = time_sec;
                            }
                            (time_sec, v)
                        })
                        .collect();

                    match property.as_str() {
                        "translation" => {
                            let target = format!("node_{bone_idx}.translation.{suffix}");
                            channels.push(AnimationChannel {
                                target_property: target,
                                keyframes: keyframes
                                    .iter()
                                    .map(|&(time, value)| Keyframe {
                                        time,
                                        value,
                                        easing: EasingFunction::Linear,
                                    })
                                    .collect(),
                            });
                        }
                        "rotation" => {
                            // Accumulate Euler curves; we'll convert to
                            // quaternion after collecting all 3 axes.
                            let entry = bone_euler_curves
                                .entry(bone_idx)
                                .or_insert_with(|| [Vec::new(), Vec::new(), Vec::new()]);
                            entry[comp_idx] = keyframes;
                        }
                        "scale" => {
                            let target = format!("node_{bone_idx}.scale.{suffix}");
                            channels.push(AnimationChannel {
                                target_property: target,
                                keyframes: keyframes
                                    .iter()
                                    .map(|&(time, value)| Keyframe {
                                        time,
                                        value,
                                        easing: EasingFunction::Linear,
                                    })
                                    .collect(),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        // Convert accumulated Euler rotation curves to quaternion channels.
        for (&bone_idx, euler_curves) in &bone_euler_curves {
            let quat_channels =
                euler_curves_to_quat_channels(bone_idx, euler_curves, &mut max_time);
            channels.extend(quat_channels);
        }

        if !channels.is_empty() {
            animations.push(KeyframeAnimation::new(
                stack_name.clone(),
                max_time,
                channels,
            ));
        }
    }

    animations
}

// ===========================================================================
// Animation helpers
// ===========================================================================

/// Finds which bone a CurveNode targets via OP connections.
///
/// Returns (bone_index, property_type) where property_type is one of
/// "translation", "rotation", "scale".
fn find_curve_node_target(
    cn_id: i64,
    conns: &FbxConnections,
    model_id_to_bone_idx: &HashMap<i64, usize>,
) -> Option<(usize, String)> {
    // CurveNode connects to a bone Model via OP connection with property
    // "Lcl Translation", "Lcl Rotation", or "Lcl Scaling".
    let parents = conns.parents_of.get(&cn_id)?;
    for &dst in parents {
        if let Some(&bone_idx) = model_id_to_bone_idx.get(&dst) {
            if let Some(prop) = conns.properties.get(&(cn_id, dst)) {
                let property_type = match prop.as_str() {
                    "Lcl Translation" => "translation",
                    "Lcl Rotation" => "rotation",
                    "Lcl Scaling" => "scale",
                    _ => continue,
                };
                return Some((bone_idx, property_type.to_string()));
            }
        }
    }
    None
}

// ===========================================================================
// Euler -> Quaternion conversion
// ===========================================================================

/// Converts three Euler rotation curves (X, Y, Z in degrees) into four
/// quaternion channels (x, y, z, w) for a single bone.
fn euler_curves_to_quat_channels(
    bone_idx: usize,
    euler: &[Vec<(f32, f32)>; 3],
    max_time: &mut f32,
) -> Vec<AnimationChannel> {
    // Merge all unique timestamps from the three Euler curves.
    let mut times: Vec<f32> = Vec::new();
    for curve in euler {
        for &(t, _) in curve {
            if !times.iter().any(|&existing| (existing - t).abs() < 1e-6) {
                times.push(t);
            }
        }
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    if times.is_empty() {
        return Vec::new();
    }

    // At each timestamp, interpolate all 3 Euler curves and convert to quaternion.
    let mut kf_x = Vec::with_capacity(times.len());
    let mut kf_y = Vec::with_capacity(times.len());
    let mut kf_z = Vec::with_capacity(times.len());
    let mut kf_w = Vec::with_capacity(times.len());

    for &t in &times {
        let rx = lerp_curve_at(t, &euler[0]);
        let ry = lerp_curve_at(t, &euler[1]);
        let rz = lerp_curve_at(t, &euler[2]);

        let [qx, qy, qz, qw] = euler_xyz_to_quat(rx, ry, rz);

        if t > *max_time {
            *max_time = t;
        }

        let easing = EasingFunction::Linear;
        kf_x.push(Keyframe {
            time: t,
            value: qx,
            easing: easing.clone(),
        });
        kf_y.push(Keyframe {
            time: t,
            value: qy,
            easing: easing.clone(),
        });
        kf_z.push(Keyframe {
            time: t,
            value: qz,
            easing: easing.clone(),
        });
        kf_w.push(Keyframe {
            time: t,
            value: qw,
            easing,
        });
    }

    vec![
        AnimationChannel {
            target_property: format!("node_{bone_idx}.rotation.x"),
            keyframes: kf_x,
        },
        AnimationChannel {
            target_property: format!("node_{bone_idx}.rotation.y"),
            keyframes: kf_y,
        },
        AnimationChannel {
            target_property: format!("node_{bone_idx}.rotation.z"),
            keyframes: kf_z,
        },
        AnimationChannel {
            target_property: format!("node_{bone_idx}.rotation.w"),
            keyframes: kf_w,
        },
    ]
}

/// Linearly interpolates a curve's value at a given time.
pub(super) fn lerp_curve_at(t: f32, curve: &[(f32, f32)]) -> f32 {
    if curve.is_empty() {
        return 0.0;
    }
    if curve.len() == 1 || t <= curve[0].0 {
        return curve[0].1;
    }
    if t >= curve.last().unwrap().0 {
        return curve.last().unwrap().1;
    }
    // Binary search for the bracketing keyframes.
    let idx = curve.partition_point(|&(ct, _)| ct < t);
    if idx == 0 {
        return curve[0].1;
    }
    let (t0, v0) = curve[idx - 1];
    let (t1, v1) = curve[idx];
    let dt = t1 - t0;
    if dt.abs() < 1e-9 {
        return v1;
    }
    let frac = (t - t0) / dt;
    v0 + frac * (v1 - v0)
}

/// Converts Euler XYZ rotation (degrees) to quaternion (x, y, z, w).
///
/// Uses FBX's default rotation order: intrinsic X-Y-Z, which corresponds
/// to the matrix multiplication R = Rx * Ry * Rz.
pub(super) fn euler_xyz_to_quat(rx_deg: f32, ry_deg: f32, rz_deg: f32) -> [f32; 4] {
    let rx = rx_deg.to_radians() * 0.5;
    let ry = ry_deg.to_radians() * 0.5;
    let rz = rz_deg.to_radians() * 0.5;

    let (sx, cx) = rx.sin_cos();
    let (sy, cy) = ry.sin_cos();
    let (sz, cz) = rz.sin_cos();

    // Q = Qx * Qy * Qz (FBX eEulerXYZ)
    [
        sx * cy * cz + cx * sy * sz, // x
        cx * sy * cz - sx * cy * sz, // y
        cx * cy * sz + sx * sy * cz, // z
        cx * cy * cz - sx * sy * sz, // w
    ]
}
