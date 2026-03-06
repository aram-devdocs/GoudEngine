//! Spatial audio helpers: distance-based attenuation models.

use crate::core::math::Vec2;

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

/// Computes attenuation volume for a spatial audio source at the given positions.
///
/// # Arguments
///
/// * `source_position` - Position of the audio source in world space
/// * `listener_position` - Position of the listener
/// * `max_distance` - Maximum audible distance
/// * `rolloff` - Rolloff exponent (1.0 = linear, 2.0 = quadratic)
///
/// # Returns
///
/// Attenuation factor (0.0-1.0)
pub(super) fn spatial_attenuation(
    source_position: Vec2,
    listener_position: Vec2,
    max_distance: f32,
    rolloff: f32,
) -> f32 {
    let distance = (source_position - listener_position).length();
    compute_attenuation_linear(distance, max_distance, rolloff)
}
