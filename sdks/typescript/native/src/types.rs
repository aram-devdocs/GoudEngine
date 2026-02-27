use goud_engine::core::math::{
    Color as EngineColor, Rect as EngineRect, Vec2 as EngineVec2, Vec3 as EngineVec3,
};
use napi_derive::napi;

#[napi(object)]
#[derive(Clone, Debug)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl From<EngineVec2> for Vec2 {
    fn from(v: EngineVec2) -> Self {
        Self {
            x: v.x as f64,
            y: v.y as f64,
        }
    }
}

impl From<&Vec2> for EngineVec2 {
    fn from(v: &Vec2) -> Self {
        Self::new(v.x as f32, v.y as f32)
    }
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<EngineVec3> for Vec3 {
    fn from(v: EngineVec3) -> Self {
        Self {
            x: v.x as f64,
            y: v.y as f64,
            z: v.z as f64,
        }
    }
}

impl From<&Vec3> for EngineVec3 {
    fn from(v: &Vec3) -> Self {
        Self::new(v.x as f32, v.y as f32, v.z as f32)
    }
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl From<EngineColor> for Color {
    fn from(c: EngineColor) -> Self {
        Self {
            r: c.r as f64,
            g: c.g as f64,
            b: c.b as f64,
            a: c.a as f64,
        }
    }
}

impl From<&Color> for EngineColor {
    fn from(c: &Color) -> Self {
        Self::rgba(c.r as f32, c.g as f32, c.b as f32, c.a as f32)
    }
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl From<EngineRect> for Rect {
    fn from(r: EngineRect) -> Self {
        Self {
            x: r.x as f64,
            y: r.y as f64,
            width: r.width as f64,
            height: r.height as f64,
        }
    }
}

impl From<&Rect> for EngineRect {
    fn from(r: &Rect) -> Self {
        Self::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)
    }
}

// Static factory functions for Color
#[napi]
pub fn color_white() -> Color {
    EngineColor::WHITE.into()
}

#[napi]
pub fn color_black() -> Color {
    EngineColor::BLACK.into()
}

#[napi]
pub fn color_red() -> Color {
    EngineColor::RED.into()
}

#[napi]
pub fn color_green() -> Color {
    EngineColor::GREEN.into()
}

#[napi]
pub fn color_blue() -> Color {
    EngineColor::BLUE.into()
}

#[napi]
pub fn color_yellow() -> Color {
    EngineColor::YELLOW.into()
}

#[napi]
pub fn color_transparent() -> Color {
    EngineColor::TRANSPARENT.into()
}

#[napi]
pub fn color_rgba(r: f64, g: f64, b: f64, a: f64) -> Color {
    EngineColor::rgba(r as f32, g as f32, b as f32, a as f32).into()
}

#[napi]
pub fn color_rgb(r: f64, g: f64, b: f64) -> Color {
    EngineColor::rgb(r as f32, g as f32, b as f32).into()
}

#[napi]
pub fn color_from_hex(hex: u32) -> Color {
    EngineColor::from_hex(hex).into()
}
