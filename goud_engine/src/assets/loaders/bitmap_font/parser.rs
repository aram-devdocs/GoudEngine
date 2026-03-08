//! BMFont text-format parser.
//!
//! Parses `.fnt` files in the BMFont text format into a [`BitmapFontAsset`].
//! Supports `info`, `common`, `page`, `char`, and `kerning` lines.

use std::collections::HashMap;

use super::asset::{BitmapCharInfo, BitmapFontAsset};

/// Parses a BMFont text-format string into a [`BitmapFontAsset`].
///
/// # Errors
///
/// Returns an error string if the file is malformed or missing required fields.
pub fn parse_bmfont(content: &str) -> Result<BitmapFontAsset, String> {
    let mut characters = HashMap::new();
    let mut kernings = HashMap::new();
    let mut texture_path = String::new();
    let mut line_height: f32 = 0.0;
    let mut base: f32 = 0.0;
    let mut scale_w: u32 = 256;
    let mut scale_h: u32 = 256;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("common ") {
            line_height = parse_value(line, "lineHeight")?;
            base = parse_value(line, "base")?;
            scale_w = parse_value(line, "scaleW")?;
            scale_h = parse_value(line, "scaleH")?;
        } else if line.starts_with("page ") {
            texture_path = parse_string_value(line, "file")?;
        } else if line.starts_with("char ") {
            let id: u32 = parse_value(line, "id")?;
            let x: u32 = parse_value(line, "x")?;
            let y: u32 = parse_value(line, "y")?;
            let width: u32 = parse_value(line, "width")?;
            let height: u32 = parse_value(line, "height")?;
            let x_offset: f32 = parse_value(line, "xoffset")?;
            let y_offset: f32 = parse_value(line, "yoffset")?;
            let x_advance: f32 = parse_value(line, "xadvance")?;

            if let Some(ch) = char::from_u32(id) {
                characters.insert(
                    ch,
                    BitmapCharInfo {
                        x,
                        y,
                        width,
                        height,
                        x_offset,
                        y_offset,
                        x_advance,
                    },
                );
            }
        } else if line.starts_with("kerning ") {
            let first_id: u32 = parse_value(line, "first")?;
            let second_id: u32 = parse_value(line, "second")?;
            let amount: f32 = parse_value(line, "amount")?;

            if let (Some(first), Some(second)) =
                (char::from_u32(first_id), char::from_u32(second_id))
            {
                kernings.insert((first, second), amount);
            }
        }
        // `info` and `chars count` lines are informational and skipped.
    }

    if texture_path.is_empty() {
        return Err("BMFont file missing 'page' line with texture file".to_string());
    }

    Ok(BitmapFontAsset {
        characters,
        texture_path,
        line_height,
        base,
        kernings,
        scale_w,
        scale_h,
        texture_data: None,
    })
}

/// Extracts a numeric value for a given key from a BMFont line.
///
/// BMFont lines have the format: `tag key=value key=value ...`
fn parse_value<T: std::str::FromStr>(line: &str, key: &str) -> Result<T, String> {
    let search = format!("{key}=");
    let start = line
        .find(&search)
        .ok_or_else(|| format!("key '{key}' not found in line: {line}"))?;
    let value_start = start + search.len();
    let rest = &line[value_start..];
    let value_str = rest.split_whitespace().next().unwrap_or("");
    value_str
        .parse::<T>()
        .map_err(|_| format!("failed to parse value for '{key}': '{value_str}'"))
}

/// Extracts a quoted string value for a given key from a BMFont line.
fn parse_string_value(line: &str, key: &str) -> Result<String, String> {
    let search = format!("{key}=");
    let start = line
        .find(&search)
        .ok_or_else(|| format!("key '{key}' not found in line: {line}"))?;
    let value_start = start + search.len();
    let rest = &line[value_start..];

    if let Some(stripped) = rest.strip_prefix('"') {
        // Quoted string: extract between quotes.
        let end = stripped
            .find('"')
            .ok_or_else(|| format!("unterminated quote for '{key}'"))?;
        Ok(stripped[..end].to_string())
    } else {
        // Unquoted: take until whitespace.
        let value_str = rest.split_whitespace().next().unwrap_or("");
        Ok(value_str.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_fnt() -> &'static str {
        include_str!("../../../../test_assets/fonts/test_bitmap.fnt")
    }

    #[test]
    fn test_parse_bmfont_char_count() {
        let asset = parse_bmfont(sample_fnt()).expect("parse should succeed");
        assert_eq!(asset.char_count(), 10, "expected 10 characters");
    }

    #[test]
    fn test_parse_bmfont_specific_char() {
        let asset = parse_bmfont(sample_fnt()).expect("parse should succeed");
        let info = asset.char_info('A').expect("'A' should be in font");

        assert_eq!(info.x, 0);
        assert_eq!(info.y, 0);
        assert_eq!(info.width, 20);
        assert_eq!(info.height, 24);
        assert_eq!(info.x_advance, 18.0);
    }

    #[test]
    fn test_parse_bmfont_kerning() {
        let asset = parse_bmfont(sample_fnt()).expect("parse should succeed");
        let k = asset.kerning('A', 'V');
        assert!(
            (k - (-2.0)).abs() < f32::EPSILON,
            "expected kerning of -2.0 for AV, got {}",
            k
        );
    }

    #[test]
    fn test_parse_bmfont_no_kerning() {
        let asset = parse_bmfont(sample_fnt()).expect("parse should succeed");
        let k = asset.kerning('Z', 'Z');
        assert_eq!(k, 0.0, "missing kerning pair should return 0.0");
    }

    #[test]
    fn test_parse_bmfont_line_height() {
        let asset = parse_bmfont(sample_fnt()).expect("parse should succeed");
        assert_eq!(asset.line_height, 32.0);
        assert_eq!(asset.base, 26.0);
    }

    #[test]
    fn test_parse_bmfont_texture_path() {
        let asset = parse_bmfont(sample_fnt()).expect("parse should succeed");
        assert_eq!(asset.texture_path, "test_bitmap.png");
    }

    #[test]
    fn test_parse_bmfont_missing_page_errors() {
        let bad_input = "info face=\"Test\" size=32\ncommon lineHeight=32 base=26\n";
        let result = parse_bmfont(bad_input);
        assert!(result.is_err());
    }
}
