use super::types::{Light, LightType, Object3D};
use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, Vector3, Vector4};
use std::collections::HashMap;

/// CPU-built directional shadow map used as a compatibility fallback.
///
/// This path rasterizes shadow depth on the CPU, so it is intentionally scoped to moderate scene
/// sizes and serves as a bridge until the renderer grows a dedicated GPU shadow pass.
pub(super) struct SoftwareShadowMap {
    pub(super) size: u32,
    pub(super) light_space_matrix: Matrix4<f32>,
    #[cfg(test)]
    pub(super) depth_values: Vec<f32>,
    pub(super) rgba8: Vec<u8>,
}

/// Maximum total vertex count for CPU shadow rasterization.
///
/// Scenes exceeding this threshold skip the software shadow pass entirely.
/// The CPU rasterizer is a compatibility bridge; once a GPU shadow pass exists
/// this limit can be removed.
const MAX_SHADOW_VERTICES: usize = 10_000;

pub(super) fn build_directional_shadow_map(
    objects: &HashMap<u32, Object3D>,
    lights: &HashMap<u32, Light>,
    size: u32,
) -> Option<SoftwareShadowMap> {
    let light = lights
        .values()
        .find(|light| light.enabled && light.light_type == LightType::Directional)?;
    let direction = if light.direction.magnitude2() > 0.0 {
        light.direction.normalize()
    } else {
        Vector3::new(0.0, -1.0, 0.0)
    };

    // Count total vertices across all objects. Each vertex uses 8 floats
    // (pos + normal + uv), so divide by the stride to get the vertex count.
    let total_vertex_floats: usize = objects.values().map(|o| o.vertices.len()).sum();
    let total_vertices = total_vertex_floats / 8;
    if total_vertices > MAX_SHADOW_VERTICES {
        return None;
    }

    let meshes = objects
        .values()
        .filter(|object| !object.vertices.is_empty())
        .map(|object| ShadowMesh {
            vertices: &object.vertices,
            model: super::core::Renderer3D::create_model_matrix(
                object.position,
                object.rotation,
                object.scale,
            ),
        })
        .collect::<Vec<_>>();
    if meshes.is_empty() {
        return None;
    }

    Some(build_shadow_map_from_meshes(
        &meshes,
        direction,
        size.max(32),
    ))
}

#[cfg(test)]
pub(super) fn sample_shadow_factor(
    shadow_map: &SoftwareShadowMap,
    world_position: Vector3<f32>,
    bias: f32,
) -> f32 {
    let projected = shadow_map.light_space_matrix * world_position.extend(1.0);
    if projected.w.abs() <= f32::EPSILON {
        return 0.0;
    }
    let ndc = projected.truncate() / projected.w;
    let uv_x = ndc.x * 0.5 + 0.5;
    let uv_y = ndc.y * 0.5 + 0.5;
    let depth = ndc.z * 0.5 + 0.5;
    if !(0.0..=1.0).contains(&uv_x) || !(0.0..=1.0).contains(&uv_y) || !(0.0..=1.0).contains(&depth)
    {
        return 0.0;
    }

    let size = shadow_map.size as usize;
    let x = (uv_x * (shadow_map.size.saturating_sub(1)) as f32).round() as usize;
    let y = (uv_y * (shadow_map.size.saturating_sub(1)) as f32).round() as usize;
    let idx = y.min(size - 1) * size + x.min(size - 1);
    let closest_depth = shadow_map.depth_values[idx];
    if closest_depth >= 1.0 {
        0.0
    } else if depth - bias > closest_depth {
        1.0
    } else {
        0.0
    }
}

pub(super) struct ShadowMesh<'a> {
    pub(super) vertices: &'a [f32],
    pub(super) model: Matrix4<f32>,
}

pub(super) fn build_shadow_map_from_meshes(
    meshes: &[ShadowMesh<'_>],
    light_direction: Vector3<f32>,
    size: u32,
) -> SoftwareShadowMap {
    let world_positions = meshes
        .iter()
        .flat_map(|mesh| world_positions(mesh.vertices, mesh.model))
        .collect::<Vec<_>>();

    let (scene_min, scene_max) = bounds(&world_positions);
    let center = (scene_min + scene_max) * 0.5;
    let light_target = Point3::from_vec(center);
    let light_origin = Point3::from_vec(center - light_direction.normalize() * 20.0);
    let up = if light_direction.y.abs() > 0.95 {
        Vector3::new(0.0, 0.0, 1.0)
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    };
    let light_view = Matrix4::look_at_rh(light_origin, light_target, up);

    let light_space_points = world_positions
        .iter()
        .map(|position| {
            let p = light_view * position.extend(1.0);
            p.truncate()
        })
        .collect::<Vec<_>>();
    let (light_min, light_max) = bounds(&light_space_points);
    let projection = cgmath::ortho(
        light_min.x,
        light_max.x,
        light_min.y,
        light_max.y,
        -light_max.z - 10.0,
        -light_min.z + 10.0,
    );
    let light_space_matrix = projection * light_view;

    let mut depth_values = vec![1.0f32; (size * size) as usize];
    for mesh in meshes {
        for triangle in mesh.vertices.chunks_exact(24) {
            let v0 = mesh.model * Vector4::new(triangle[0], triangle[1], triangle[2], 1.0);
            let v1 = mesh.model * Vector4::new(triangle[8], triangle[9], triangle[10], 1.0);
            let v2 = mesh.model * Vector4::new(triangle[16], triangle[17], triangle[18], 1.0);
            rasterize_triangle(
                &mut depth_values,
                size,
                light_space_matrix,
                v0.truncate(),
                v1.truncate(),
                v2.truncate(),
            );
        }
    }

    let rgba8 = depth_values
        .iter()
        .flat_map(|depth| {
            let byte = (depth.clamp(0.0, 1.0) * 255.0) as u8;
            [byte, byte, byte, 255]
        })
        .collect();

    SoftwareShadowMap {
        size,
        light_space_matrix,
        #[cfg(test)]
        depth_values,
        rgba8,
    }
}

fn world_positions(vertices: &[f32], model: Matrix4<f32>) -> Vec<Vector3<f32>> {
    vertices
        .chunks_exact(8)
        .map(|vertex| {
            let world = model * Vector4::new(vertex[0], vertex[1], vertex[2], 1.0);
            world.truncate()
        })
        .collect()
}

fn bounds(points: &[Vector3<f32>]) -> (Vector3<f32>, Vector3<f32>) {
    let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for point in points {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
    }
    (min, max)
}

fn rasterize_triangle(
    depth_values: &mut [f32],
    size: u32,
    light_space_matrix: Matrix4<f32>,
    p0: Vector3<f32>,
    p1: Vector3<f32>,
    p2: Vector3<f32>,
) {
    let clip0 = light_space_matrix * p0.extend(1.0);
    let clip1 = light_space_matrix * p1.extend(1.0);
    let clip2 = light_space_matrix * p2.extend(1.0);
    if clip0.w.abs() <= f32::EPSILON
        || clip1.w.abs() <= f32::EPSILON
        || clip2.w.abs() <= f32::EPSILON
    {
        return;
    }

    let ndc0 = clip0.truncate() / clip0.w;
    let ndc1 = clip1.truncate() / clip1.w;
    let ndc2 = clip2.truncate() / clip2.w;

    let s0 = ndc_to_screen(ndc0, size);
    let s1 = ndc_to_screen(ndc1, size);
    let s2 = ndc_to_screen(ndc2, size);

    let min_x = s0.0.min(s1.0).min(s2.0).floor().max(0.0) as u32;
    let max_x = s0.0.max(s1.0).max(s2.0).ceil().min((size - 1) as f32) as u32;
    let min_y = s0.1.min(s1.1).min(s2.1).floor().max(0.0) as u32;
    let max_y = s0.1.max(s1.1).max(s2.1).ceil().min((size - 1) as f32) as u32;

    let area = edge((s0.0, s0.1), (s1.0, s1.1), (s2.0, s2.1));
    if area.abs() <= f32::EPSILON {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let sample = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge((s1.0, s1.1), (s2.0, s2.1), sample) / area;
            let w1 = edge((s2.0, s2.1), (s0.0, s0.1), sample) / area;
            let w2 = edge((s0.0, s0.1), (s1.0, s1.1), sample) / area;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let depth = ((w0 * ndc0.z + w1 * ndc1.z + w2 * ndc2.z) * 0.5 + 0.5).clamp(0.0, 1.0);
            let idx = (y * size + x) as usize;
            depth_values[idx] = depth_values[idx].min(depth);
        }
    }
}

fn ndc_to_screen(ndc: Vector3<f32>, size: u32) -> (f32, f32) {
    let span = (size.saturating_sub(1)) as f32;
    ((ndc.x * 0.5 + 0.5) * span, (ndc.y * 0.5 + 0.5) * span)
}

fn edge(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    (c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)
}
