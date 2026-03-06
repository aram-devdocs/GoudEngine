//! Trait implementations for [`GlobalTransform2D`]: Default, Debug, From, Mul, Component.

use crate::ecs::components::transform2d::{Mat3x3, Transform2D};
use crate::ecs::Component;
use std::fmt;

use super::types::GlobalTransform2D;

impl Default for GlobalTransform2D {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl fmt::Debug for GlobalTransform2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t, r, s) = self.decompose();
        f.debug_struct("GlobalTransform2D")
            .field("translation", &(t.x, t.y))
            .field("rotation", &format!("{:.3} rad", r))
            .field("scale", &(s.x, s.y))
            .finish()
    }
}

impl From<Transform2D> for GlobalTransform2D {
    /// Converts a local Transform2D to a GlobalTransform2D.
    ///
    /// This is used for root entities where local == global.
    #[inline]
    fn from(transform: Transform2D) -> Self {
        GlobalTransform2D {
            matrix: transform.matrix(),
        }
    }
}

impl From<&Transform2D> for GlobalTransform2D {
    #[inline]
    fn from(transform: &Transform2D) -> Self {
        GlobalTransform2D {
            matrix: transform.matrix(),
        }
    }
}

impl From<Mat3x3> for GlobalTransform2D {
    #[inline]
    fn from(matrix: Mat3x3) -> Self {
        GlobalTransform2D { matrix }
    }
}

// Implement Component trait
impl Component for GlobalTransform2D {}

// Implement multiplication operators
impl std::ops::Mul for GlobalTransform2D {
    type Output = GlobalTransform2D;

    #[inline]
    fn mul(self, rhs: GlobalTransform2D) -> GlobalTransform2D {
        self.mul_transform(&rhs)
    }
}

impl std::ops::Mul<&GlobalTransform2D> for GlobalTransform2D {
    type Output = GlobalTransform2D;

    #[inline]
    fn mul(self, rhs: &GlobalTransform2D) -> GlobalTransform2D {
        self.mul_transform(rhs)
    }
}

impl std::ops::Mul<Transform2D> for GlobalTransform2D {
    type Output = GlobalTransform2D;

    #[inline]
    fn mul(self, rhs: Transform2D) -> GlobalTransform2D {
        self.transform_by(&rhs)
    }
}

impl std::ops::Mul<&Transform2D> for GlobalTransform2D {
    type Output = GlobalTransform2D;

    #[inline]
    fn mul(self, rhs: &Transform2D) -> GlobalTransform2D {
        self.transform_by(rhs)
    }
}
