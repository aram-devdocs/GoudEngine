use crate::core::math::{Rect, Vec2};
use crate::rendering::RenderViewport;
use cgmath::{ortho, Vector4};

#[derive(Clone, Copy, Debug)]
struct FrustumPlane2D {
    a: f32,
    b: f32,
    d: f32,
}

impl FrustumPlane2D {
    fn from_clip_coefficients(coefficients: Vector4<f32>) -> Self {
        let length = (coefficients.x * coefficients.x + coefficients.y * coefficients.y).sqrt();
        if length <= f32::EPSILON {
            return Self {
                a: 0.0,
                b: 0.0,
                d: 0.0,
            };
        }
        Self {
            a: coefficients.x / length,
            b: coefficients.y / length,
            d: coefficients.w / length,
        }
    }

    fn signed_distance(self, point: Vec2) -> f32 {
        self.a * point.x + self.b * point.y + self.d
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SpriteFrustum2D {
    planes: [FrustumPlane2D; 4],
}

impl SpriteFrustum2D {
    pub(super) fn from_viewport(viewport: RenderViewport) -> Self {
        let projection = ortho(
            0.0,
            viewport.logical_width.max(1) as f32,
            viewport.logical_height.max(1) as f32,
            0.0,
            -1.0,
            1.0,
        );
        let row0 = Vector4::new(
            projection.x.x,
            projection.y.x,
            projection.z.x,
            projection.w.x,
        );
        let row1 = Vector4::new(
            projection.x.y,
            projection.y.y,
            projection.z.y,
            projection.w.y,
        );
        let row3 = Vector4::new(
            projection.x.w,
            projection.y.w,
            projection.z.w,
            projection.w.w,
        );
        Self {
            planes: [
                FrustumPlane2D::from_clip_coefficients(row3 + row0),
                FrustumPlane2D::from_clip_coefficients(row3 - row0),
                FrustumPlane2D::from_clip_coefficients(row3 + row1),
                FrustumPlane2D::from_clip_coefficients(row3 - row1),
            ],
        }
    }

    pub(super) fn intersects_rect(self, rect: &Rect) -> bool {
        let min = rect.min();
        let max = rect.max();
        self.planes.iter().all(|plane| {
            let positive = Vec2::new(
                if plane.a >= 0.0 { max.x } else { min.x },
                if plane.b >= 0.0 { max.y } else { min.y },
            );
            plane.signed_distance(positive) >= 0.0
        })
    }
}
