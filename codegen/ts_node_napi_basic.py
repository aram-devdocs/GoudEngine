#!/usr/bin/env python3
"""Basic napi-rs generation helpers for the TypeScript Node SDK."""

from ts_node_shared import NATIVE_SRC, RUST_HEADER, schema, to_snake, write_generated


def gen_napi_rust_types():
    lines = [RUST_HEADER, "use napi_derive::napi;", ""]
    struct_meta = {
        "Vec2": {"fields": [("x", "f64"), ("y", "f64")]},
        "Vec3": {"fields": [("x", "f64"), ("y", "f64"), ("z", "f64")]},
        "Color": {"fields": [("r", "f64"), ("g", "f64"), ("b", "f64"), ("a", "f64")]},
        "Rect": {"fields": [("x", "f64"), ("y", "f64"), ("width", "f64"), ("height", "f64")]},
    }
    for name, meta in struct_meta.items():
        lines += ["#[napi(object)]", "#[derive(Clone, Debug)]", f"pub struct {name} {{"]
        for fname, ftype in meta["fields"]:
            lines.append(f"    pub {fname}: {ftype},")
        lines += ["}", ""]

    for factory in schema["types"]["Color"].get("factories", []):
        fname = factory["name"]
        fargs = factory.get("args", [])
        val = factory.get("value")
        fn_name = f"color_{to_snake(fname)}"
        if fname == "rgba":
            lines += ["#[napi]", f"pub fn {fn_name}(r: f64, g: f64, b: f64, a: f64) -> Color {{", "    Color { r, g, b, a }", "}"]
        elif fname == "rgb":
            lines += ["#[napi]", f"pub fn {fn_name}(r: f64, g: f64, b: f64) -> Color {{", "    Color { r, g, b, a: 1.0 }", "}"]
        elif fname == "fromHex":
            lines += ["#[napi]", f"pub fn {fn_name}(hex: u32) -> Color {{", "    Color {", "        r: ((hex >> 16) & 0xFF) as f64 / 255.0,", "        g: ((hex >> 8) & 0xFF) as f64 / 255.0,", "        b: (hex & 0xFF) as f64 / 255.0,", "        a: 1.0,", "    }", "}"]
        elif fname == "fromU8":
            lines += ["#[napi]", f"pub fn {fn_name}(r: u32, g: u32, b: u32, a: u32) -> Color {{", "    Color {", "        r: r as f64 / 255.0,", "        g: g as f64 / 255.0,", "        b: b as f64 / 255.0,", "        a: a as f64 / 255.0,", "    }", "}"]
        elif val is not None and not fargs:
            lines += ["#[napi]", f"pub fn {fn_name}() -> Color {{", f"    Color {{ r: {float(val[0])}, g: {float(val[1])}, b: {float(val[2])}, a: {float(val[3])} }}", "}"]
        lines.append("")

    write_generated(NATIVE_SRC / "types.g.rs", "\n".join(lines))


def gen_napi_rust_entity():
    lines = [
        RUST_HEADER,
        "use napi::bindgen_prelude::*;",
        "use napi_derive::napi;",
        "",
        "/// PLACEHOLDER entity bits: index=u32::MAX, generation=0.",
        "const PLACEHOLDER_BITS: u64 = u32::MAX as u64;",
        "",
        "#[napi]",
        "pub struct Entity {",
        "    /// Packed entity bits: (generation << 32) | index.",
        "    pub(crate) bits: u64,",
        "}",
        "",
        "#[napi]",
        "impl Entity {",
        "    #[napi(constructor)]",
        "    pub fn new(index: u32, generation: u32) -> Self {",
        "        Self { bits: ((generation as u64) << 32) | (index as u64) }",
        "    }",
        "",
        "    #[napi(factory)]",
        "    pub fn placeholder() -> Self {",
        "        Self { bits: PLACEHOLDER_BITS }",
        "    }",
        "",
        "    #[napi(factory)]",
        "    pub fn from_bits(bits: BigInt) -> Result<Self> {",
        "        let (_, value, _) = bits.get_u64();",
        "        Ok(Self { bits: value })",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn index(&self) -> u32 { self.bits as u32 }",
        "",
        "    #[napi(getter)]",
        "    pub fn generation(&self) -> u32 { (self.bits >> 32) as u32 }",
        "",
        "    #[napi(getter)]",
        "    pub fn is_placeholder(&self) -> bool { self.bits == PLACEHOLDER_BITS }",
        "",
        "    #[napi]",
        "    pub fn to_bits(&self) -> BigInt { BigInt::from(self.bits) }",
        "",
        "    #[napi]",
        "    pub fn display(&self) -> String {",
        "        let index = self.bits as u32;",
        "        let gen = (self.bits >> 32) as u32;",
        '        format!("Entity({}:{})", index, gen)',
        "    }",
        "}",
        "",
    ]
    write_generated(NATIVE_SRC / "entity.g.rs", "\n".join(lines))


def gen_napi_rust_components():
    lines = [
        RUST_HEADER,
        "use crate::types::Color;",
        "use napi_derive::napi;",
        "",
        "// =============================================================================",
        "// Transform2D",
        "// =============================================================================",
        "",
        "#[napi(object)]",
        "#[derive(Clone, Debug)]",
        "pub struct Transform2DData {",
        "    pub position_x: f64,",
        "    pub position_y: f64,",
        "    pub rotation: f64,",
        "    pub scale_x: f64,",
        "    pub scale_y: f64,",
        "}",
        "",
        "impl Default for Transform2DData {",
        "    fn default() -> Self {",
        "        Self { position_x: 0.0, position_y: 0.0, rotation: 0.0, scale_x: 1.0, scale_y: 1.0 }",
        "    }",
        "}",
        "",
        "// =============================================================================",
        "// Sprite",
        "// =============================================================================",
        "",
        "#[napi(object)]",
        "#[derive(Clone, Debug)]",
        "pub struct SpriteData {",
        "    pub color: Color,",
        "    pub flip_x: bool,",
        "    pub flip_y: bool,",
        "    pub anchor_x: f64,",
        "    pub anchor_y: f64,",
        "    pub custom_width: Option<f64>,",
        "    pub custom_height: Option<f64>,",
        "    pub source_rect_x: Option<f64>,",
        "    pub source_rect_y: Option<f64>,",
        "    pub source_rect_width: Option<f64>,",
        "    pub source_rect_height: Option<f64>,",
        "}",
        "",
        "impl Default for SpriteData {",
        "    fn default() -> Self {",
        "        Self {",
        "            color: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },",
        "            flip_x: false, flip_y: false, anchor_x: 0.5, anchor_y: 0.5,",
        "            custom_width: None, custom_height: None,",
        "            source_rect_x: None, source_rect_y: None, source_rect_width: None, source_rect_height: None,",
        "        }",
        "    }",
        "}",
        "",
        "#[napi]",
        "pub fn transform2d_default() -> Transform2DData { Transform2DData::default() }",
        "",
        "#[napi]",
        "pub fn transform2d_from_position(x: f64, y: f64) -> Transform2DData {",
        "    Transform2DData { position_x: x, position_y: y, ..Transform2DData::default() }",
        "}",
        "",
        "#[napi]",
        "pub fn transform2d_from_scale(x: f64, y: f64) -> Transform2DData {",
        "    Transform2DData { scale_x: x, scale_y: y, ..Transform2DData::default() }",
        "}",
        "",
        "#[napi]",
        "pub fn transform2d_from_rotation(radians: f64) -> Transform2DData {",
        "    Transform2DData { rotation: radians, ..Transform2DData::default() }",
        "}",
        "",
        "#[napi]",
        "pub fn sprite_default() -> SpriteData { SpriteData::default() }",
        "",
        "#[napi]",
        "pub fn vec2(x: f64, y: f64) -> crate::types::Vec2 { crate::types::Vec2 { x, y } }",
        "",
        "#[napi]",
        "pub fn vec2_zero() -> crate::types::Vec2 { crate::types::Vec2 { x: 0.0, y: 0.0 } }",
        "",
        "#[napi]",
        "pub fn vec2_one() -> crate::types::Vec2 { crate::types::Vec2 { x: 1.0, y: 1.0 } }",
        "",
    ]
    write_generated(NATIVE_SRC / "components.g.rs", "\n".join(lines))
