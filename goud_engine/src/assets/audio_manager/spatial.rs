//! Spatial audio helpers: distance-based attenuation models.

/// Computes linear attenuation based on distance.
///
/// Formula: attenuation = max(0, 1 - (distance / max_distance) ^ rolloff)
///
/// # Arguments
///
/// * `distance` - Distance from listener to source
/// * `max_distance` - Maximum audible distance (0 volume beyond)
/// * `rolloff` - Rolloff exponent (1.0 = linear, 2.0 = quadratic, etc.)
///
/// # Returns
///
/// Attenuation factor (0.0-1.0)
pub(super) fn compute_attenuation_linear(distance: f32, max_distance: f32, rolloff: f32) -> f32 {
    if max_distance <= 0.0 {
        return 1.0; // No attenuation
    }

    if distance >= max_distance {
        return 0.0; // Beyond max distance
    }

    // Linear falloff with rolloff factor
    let normalized_distance = distance / max_distance;
    let attenuation = 1.0 - normalized_distance.powf(rolloff);
    attenuation.max(0.0)
}

/// Computes inverse distance attenuation (realistic physics-based falloff).
///
/// Formula: attenuation = reference_distance / (reference_distance + rolloff * (distance - reference_distance))
///
/// # Arguments
///
/// * `distance` - Distance from listener to source
/// * `max_distance` - Maximum audible distance (used for clamping)
/// * `rolloff` - Rolloff factor (1.0 = realistic, higher = faster falloff)
///
/// # Returns
///
/// Attenuation factor (0.0-1.0)
#[cfg(test)]
pub(super) fn compute_attenuation_inverse(distance: f32, max_distance: f32, rolloff: f32) -> f32 {
    if max_distance <= 0.0 {
        return 1.0;
    }

    if distance >= max_distance {
        return 0.0;
    }

    // Inverse distance attenuation (reference distance = 1.0)
    let reference_distance = 1.0;
    let attenuation = reference_distance
        / (reference_distance + rolloff * (distance - reference_distance).max(0.0));
    attenuation.clamp(0.0, 1.0)
}

/// Computes exponential attenuation (dramatic falloff).
///
/// Formula: attenuation = (1 - distance / max_distance) ^ rolloff
///
/// # Arguments
///
/// * `distance` - Distance from listener to source
/// * `max_distance` - Maximum audible distance
/// * `rolloff` - Rolloff exponent (higher = more dramatic falloff)
///
/// # Returns
///
/// Attenuation factor (0.0-1.0)
#[cfg(test)]
pub(super) fn compute_attenuation_exponential(
    distance: f32,
    max_distance: f32,
    rolloff: f32,
) -> f32 {
    if max_distance <= 0.0 {
        return 1.0;
    }

    if distance >= max_distance {
        return 0.0;
    }

    // Exponential falloff
    let normalized_distance = distance / max_distance;
    let attenuation = (1.0 - normalized_distance).powf(rolloff);
    attenuation.max(0.0)
}

/// Computes attenuation volume for a 3D spatial audio source.
pub(super) fn spatial_attenuation_3d(
    source_position: [f32; 3],
    listener_position: [f32; 3],
    max_distance: f32,
    rolloff: f32,
) -> f32 {
    let dx = source_position[0] - listener_position[0];
    let dy = source_position[1] - listener_position[1];
    let dz = source_position[2] - listener_position[2];
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();
    compute_attenuation_linear(distance, max_distance, rolloff)
}

#[cfg(test)]
pub(super) fn compute_stereo_pan(source_position: [f32; 3], listener_position: [f32; 3]) -> f32 {
    let x_delta = source_position[0] - listener_position[0];
    if x_delta.abs() <= f32::EPSILON {
        0.0
    } else {
        (x_delta / (x_delta.abs() + 1.0)).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
pub(super) fn stereo_gains_from_pan(pan: f32) -> (f32, f32) {
    let p = pan.clamp(-1.0, 1.0);
    let left = ((1.0 - p) * 0.5).sqrt();
    let right = ((1.0 + p) * 0.5).sqrt();
    (left, right)
}
