//! Collision detection methods for WasmGame.
//!
//! All functions are pure math — no GL context or rendering state needed.

use wasm_bindgen::prelude::*;

use crate::core::math::Vec2;
use crate::ecs::collision::{aabb_aabb_collision, circle_aabb_collision, circle_circle_collision};

use super::WasmGame;

// ---------------------------------------------------------------------------
// Contact data transfer object
// ---------------------------------------------------------------------------

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
/// Collision contact data returned by wasm collision helper methods.
pub struct WasmContact {
    /// Contact point X coordinate.
    pub point_x: f32,
    /// Contact point Y coordinate.
    pub point_y: f32,
    /// Contact normal X component.
    pub normal_x: f32,
    /// Contact normal Y component.
    pub normal_y: f32,
    /// Penetration depth along the contact normal.
    pub penetration: f32,
}

// ---------------------------------------------------------------------------
// Collision detection methods
// ---------------------------------------------------------------------------

#[wasm_bindgen]
impl WasmGame {
    /// Tests axis-aligned box vs axis-aligned box overlap and returns contact data.
    pub fn collision_aabb_aabb(
        &self,
        center_ax: f32,
        center_ay: f32,
        half_wa: f32,
        half_ha: f32,
        center_bx: f32,
        center_by: f32,
        half_wb: f32,
        half_hb: f32,
    ) -> Option<WasmContact> {
        let center_a = Vec2::new(center_ax, center_ay);
        let half_a = Vec2::new(half_wa, half_ha);
        let center_b = Vec2::new(center_bx, center_by);
        let half_b = Vec2::new(half_wb, half_hb);

        aabb_aabb_collision(center_a, half_a, center_b, half_b).map(|c| WasmContact {
            point_x: c.point.x,
            point_y: c.point.y,
            normal_x: c.normal.x,
            normal_y: c.normal.y,
            penetration: c.penetration,
        })
    }

    /// Tests circle vs circle overlap and returns contact data.
    pub fn collision_circle_circle(
        &self,
        center_ax: f32,
        center_ay: f32,
        radius_a: f32,
        center_bx: f32,
        center_by: f32,
        radius_b: f32,
    ) -> Option<WasmContact> {
        let center_a = Vec2::new(center_ax, center_ay);
        let center_b = Vec2::new(center_bx, center_by);

        circle_circle_collision(center_a, radius_a, center_b, radius_b).map(|c| WasmContact {
            point_x: c.point.x,
            point_y: c.point.y,
            normal_x: c.normal.x,
            normal_y: c.normal.y,
            penetration: c.penetration,
        })
    }

    /// Tests circle vs axis-aligned box overlap and returns contact data.
    pub fn collision_circle_aabb(
        &self,
        cx: f32,
        cy: f32,
        cr: f32,
        bx: f32,
        by: f32,
        bhw: f32,
        bhh: f32,
    ) -> Option<WasmContact> {
        let circle_center = Vec2::new(cx, cy);
        let box_center = Vec2::new(bx, by);
        let box_half = Vec2::new(bhw, bhh);

        circle_aabb_collision(circle_center, cr, box_center, box_half).map(|c| WasmContact {
            point_x: c.point.x,
            point_y: c.point.y,
            normal_x: c.normal.x,
            normal_y: c.normal.y,
            penetration: c.penetration,
        })
    }

    /// Returns whether point `(px, py)` is inside the rectangle.
    pub fn point_in_rect(&self, px: f32, py: f32, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
        px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
    }

    /// Returns whether point `(px, py)` is inside the circle.
    pub fn point_in_circle(&self, px: f32, py: f32, cx: f32, cy: f32, r: f32) -> bool {
        let dx = px - cx;
        let dy = py - cy;
        (dx * dx + dy * dy) <= (r * r)
    }

    /// Returns whether two AABBs overlap.
    pub fn aabb_overlap(
        &self,
        min_ax: f32,
        min_ay: f32,
        max_ax: f32,
        max_ay: f32,
        min_bx: f32,
        min_by: f32,
        max_bx: f32,
        max_by: f32,
    ) -> bool {
        max_ax >= min_bx && min_ax <= max_bx && max_ay >= min_by && min_ay <= max_by
    }

    /// Returns whether two circles overlap.
    pub fn circle_overlap(&self, x1: f32, y1: f32, r1: f32, x2: f32, y2: f32, r2: f32) -> bool {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let combined = r1 + r2;
        (dx * dx + dy * dy) <= (combined * combined)
    }

    /// Returns Euclidean distance between two points.
    pub fn distance(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns squared Euclidean distance between two points.
    pub fn distance_squared(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        let dx = x2 - x1;
        let dy = y2 - y1;
        dx * dx + dy * dy
    }
}
