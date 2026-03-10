use super::Aabb;
use crate::core::providers::types::ColliderDesc;

pub(super) fn ordered_pair(left: u64, right: u64) -> (u64, u64) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

pub(super) fn collider_half_extents(desc: &ColliderDesc) -> [f32; 2] {
    match desc.shape {
        0 => [desc.radius.max(0.01), desc.radius.max(0.01)],
        1 => desc.half_extents,
        2 => [desc.radius.max(0.01), desc.half_extents[1] + desc.radius],
        _ => [desc.radius.max(0.01), desc.radius.max(0.01)],
    }
}

pub(super) fn layers_interact(left: &ColliderDesc, right: &ColliderDesc) -> bool {
    left.layer & right.mask != 0 && right.layer & left.mask != 0
}

pub(super) fn overlap(left: Aabb, right: Aabb) -> Option<([f32; 2], f32)> {
    let left_center = left.center();
    let right_center = right.center();
    let left_half = left.half_extents();
    let right_half = right.half_extents();
    let delta_x = right_center[0] - left_center[0];
    let delta_y = right_center[1] - left_center[1];
    let overlap_x = left_half[0] + right_half[0] - delta_x.abs();
    let overlap_y = left_half[1] + right_half[1] - delta_y.abs();
    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        return None;
    }

    if overlap_x < overlap_y {
        let direction = if delta_x < 0.0 { -1.0 } else { 1.0 };
        Some(([direction, 0.0], overlap_x))
    } else {
        let direction = if delta_y < 0.0 { -1.0 } else { 1.0 };
        Some(([0.0, direction], overlap_y))
    }
}

pub(super) fn raycast_aabb(
    origin: [f32; 2],
    dir: [f32; 2],
    max_dist: f32,
    aabb: Aabb,
) -> Option<(f32, [f32; 2])> {
    let mut t_min = 0.0;
    let mut t_max = max_dist;
    let mut normal = [0.0, 0.0];

    for axis in 0..2 {
        if dir[axis].abs() < f32::EPSILON {
            if origin[axis] < aabb.min[axis] || origin[axis] > aabb.max[axis] {
                return None;
            }
            continue;
        }

        let inv = 1.0 / dir[axis];
        let mut near = (aabb.min[axis] - origin[axis]) * inv;
        let mut far = (aabb.max[axis] - origin[axis]) * inv;
        let axis_normal = if axis == 0 {
            [-inv.signum(), 0.0]
        } else {
            [0.0, -inv.signum()]
        };

        if near > far {
            std::mem::swap(&mut near, &mut far);
        }
        if near > t_min {
            t_min = near;
            normal = axis_normal;
        }
        t_max = t_max.min(far);
        if t_min > t_max {
            return None;
        }
    }

    (0.0..=max_dist).contains(&t_min).then_some((t_min, normal))
}

pub(super) fn circle_overlaps_aabb(center: [f32; 2], radius: f32, aabb: Aabb) -> bool {
    let closest_x = center[0].clamp(aabb.min[0], aabb.max[0]);
    let closest_y = center[1].clamp(aabb.min[1], aabb.max[1]);
    let dx = center[0] - closest_x;
    let dy = center[1] - closest_y;
    dx * dx + dy * dy <= radius * radius
}
