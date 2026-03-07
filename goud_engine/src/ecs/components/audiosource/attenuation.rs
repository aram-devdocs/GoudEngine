//! Distance-based audio attenuation models.

/// Audio attenuation model for distance-based volume falloff.
///
/// Controls how audio volume decreases with distance from the listener.
/// Different models provide different falloff curves for realistic or
/// stylized audio behavior.
///
/// # Models
///
/// - **Linear**: Linear falloff (volume = 1 - distance/max_distance)
/// - **InverseDistance**: Realistic inverse distance falloff (volume = 1 / (1 + distance))
/// - **Exponential**: Exponential falloff (volume = (1 - distance/max_distance)^rolloff)
/// - **None**: No attenuation (constant volume regardless of distance)
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::AttenuationModel;
///
/// let linear = AttenuationModel::Linear;
/// let inverse = AttenuationModel::InverseDistance;
/// let exponential = AttenuationModel::Exponential { rolloff: 2.0 };
/// let none = AttenuationModel::None;
///
/// assert_eq!(linear.name(), "Linear");
/// assert_eq!(inverse.name(), "InverseDistance");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AttenuationModel {
    /// Linear falloff: volume = max(0, 1 - distance/max_distance)
    Linear,
    /// Inverse distance falloff: volume = 1 / (1 + distance)
    InverseDistance,
    /// Exponential falloff: volume = max(0, (1 - distance/max_distance)^rolloff)
    Exponential {
        /// The exponent for the falloff curve.
        rolloff: f32,
    },
    /// No attenuation (constant volume)
    None,
}

impl AttenuationModel {
    /// Returns the model name for debugging.
    pub fn name(&self) -> &str {
        match self {
            AttenuationModel::Linear => "Linear",
            AttenuationModel::InverseDistance => "InverseDistance",
            AttenuationModel::Exponential { .. } => "Exponential",
            AttenuationModel::None => "None",
        }
    }

    /// Computes the attenuation factor (0.0-1.0) based on distance.
    ///
    /// # Arguments
    ///
    /// - `distance`: Distance from listener (must be >= 0)
    /// - `max_distance`: Maximum distance for attenuation (must be > 0)
    ///
    /// # Returns
    ///
    /// Volume multiplier in range [0.0, 1.0]
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::AttenuationModel;
    ///
    /// let model = AttenuationModel::Linear;
    /// assert_eq!(model.compute_attenuation(0.0, 100.0), 1.0);
    /// assert_eq!(model.compute_attenuation(50.0, 100.0), 0.5);
    /// assert_eq!(model.compute_attenuation(100.0, 100.0), 0.0);
    /// assert_eq!(model.compute_attenuation(150.0, 100.0), 0.0); // Beyond max
    /// ```
    pub fn compute_attenuation(&self, distance: f32, max_distance: f32) -> f32 {
        match self {
            AttenuationModel::Linear => {
                if distance >= max_distance {
                    0.0
                } else {
                    (1.0 - distance / max_distance).max(0.0)
                }
            }
            AttenuationModel::InverseDistance => 1.0 / (1.0 + distance),
            AttenuationModel::Exponential { rolloff } => {
                if distance >= max_distance {
                    0.0
                } else {
                    ((1.0 - distance / max_distance).powf(*rolloff)).max(0.0)
                }
            }
            AttenuationModel::None => 1.0,
        }
    }
}

impl Default for AttenuationModel {
    /// Returns `AttenuationModel::InverseDistance` as the default (most realistic).
    fn default() -> Self {
        AttenuationModel::InverseDistance
    }
}

impl std::fmt::Display for AttenuationModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttenuationModel::Exponential { rolloff } => {
                write!(f, "Exponential(rolloff={})", rolloff)
            }
            other => write!(f, "{}", other.name()),
        }
    }
}
