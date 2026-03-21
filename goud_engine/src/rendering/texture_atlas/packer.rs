//! Shelf-based rectangle packing algorithm for texture atlases.
//!
//! Packs rectangles into rows (shelves). Each shelf is as tall as its
//! tallest member. For best results, sort inputs by height (tallest first)
//! before packing.

/// A rectangle placement result within the atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedRect {
    /// X offset in pixels from the left edge of the atlas.
    pub x: u32,
    /// Y offset in pixels from the top edge of the atlas.
    pub y: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Shelf-based rectangle packer.
///
/// Places rectangles left-to-right within the current row.  When a
/// rectangle does not fit horizontally, a new row starts below the
/// current shelf.
#[derive(Debug, Clone)]
pub struct ShelfPacker {
    atlas_width: u32,
    atlas_height: u32,
    /// X cursor within the current shelf.
    cursor_x: u32,
    /// Y origin of the current shelf.
    cursor_y: u32,
    /// Height of the tallest item on the current shelf.
    row_height: u32,
    /// Padding between packed rectangles (pixels).
    padding: u32,
    /// Cumulative pixel area consumed by packed rectangles (excluding padding).
    used_area: u64,
}

impl ShelfPacker {
    /// Creates a new packer for an atlas of the given dimensions.
    pub fn new(width: u32, height: u32, padding: u32) -> Self {
        Self {
            atlas_width: width,
            atlas_height: height,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            padding,
            used_area: 0,
        }
    }

    /// Attempts to pack a rectangle of `width x height` pixels.
    ///
    /// Returns the placement if successful, or `None` if the rectangle
    /// does not fit in the remaining atlas space.
    pub fn pack(&mut self, width: u32, height: u32) -> Option<PackedRect> {
        if width == 0 || height == 0 {
            return None;
        }
        if width > self.atlas_width || height > self.atlas_height {
            return None;
        }

        // Does the rect fit on the current shelf?
        let padded_w = if self.cursor_x > 0 {
            self.padding + width
        } else {
            width
        };

        if self.cursor_x + padded_w > self.atlas_width {
            // Start a new shelf below the current one.
            self.cursor_y += self.row_height + self.padding;
            self.cursor_x = 0;
            self.row_height = 0;
        }

        // Check vertical space.
        if self.cursor_y + height > self.atlas_height {
            return None;
        }

        let x = if self.cursor_x > 0 {
            self.cursor_x + self.padding
        } else {
            self.cursor_x
        };

        let rect = PackedRect {
            x,
            y: self.cursor_y,
            width,
            height,
        };

        self.cursor_x = x + width;
        if height > self.row_height {
            self.row_height = height;
        }
        self.used_area += u64::from(width) * u64::from(height);

        Some(rect)
    }

    /// Total pixel area consumed by packed rectangles (excludes padding).
    pub fn used_area(&self) -> u64 {
        self.used_area
    }

    /// Total pixel area of the atlas.
    pub fn total_area(&self) -> u64 {
        u64::from(self.atlas_width) * u64::from(self.atlas_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_rect_fits() {
        let mut p = ShelfPacker::new(128, 128, 1);
        let r = p.pack(32, 32).unwrap();
        assert_eq!(
            r,
            PackedRect {
                x: 0,
                y: 0,
                width: 32,
                height: 32
            }
        );
    }

    #[test]
    fn test_two_rects_same_row() {
        let mut p = ShelfPacker::new(128, 128, 1);
        let a = p.pack(32, 32).unwrap();
        let b = p.pack(32, 32).unwrap();
        assert_eq!(a.x, 0);
        // Second rect starts after first + padding
        assert_eq!(b.x, 33);
        assert_eq!(b.y, 0);
    }

    #[test]
    fn test_new_shelf_on_overflow() {
        let mut p = ShelfPacker::new(65, 128, 1);
        let a = p.pack(32, 16).unwrap();
        let b = p.pack(32, 16).unwrap();
        // Third should go to next shelf (32 + 1 + 32 = 65 = full row)
        let c = p.pack(32, 16).unwrap();
        assert_eq!(a.y, 0);
        assert_eq!(b.y, 0);
        assert_eq!(c.y, 17); // 16 + 1 padding
    }

    #[test]
    fn test_exact_fit() {
        let mut p = ShelfPacker::new(64, 64, 0);
        let r = p.pack(64, 64).unwrap();
        assert_eq!(
            r,
            PackedRect {
                x: 0,
                y: 0,
                width: 64,
                height: 64
            }
        );
        // No more space
        assert!(p.pack(1, 1).is_none());
    }

    #[test]
    fn test_too_large_for_atlas() {
        let mut p = ShelfPacker::new(64, 64, 0);
        assert!(p.pack(65, 32).is_none());
        assert!(p.pack(32, 65).is_none());
    }

    #[test]
    fn test_zero_size_rejected() {
        let mut p = ShelfPacker::new(64, 64, 0);
        assert!(p.pack(0, 32).is_none());
        assert!(p.pack(32, 0).is_none());
    }

    #[test]
    fn test_used_area_tracking() {
        let mut p = ShelfPacker::new(128, 128, 0);
        p.pack(10, 10).unwrap();
        p.pack(20, 20).unwrap();
        assert_eq!(p.used_area(), 100 + 400);
    }

    #[test]
    fn test_vertical_overflow() {
        let mut p = ShelfPacker::new(32, 32, 1);
        p.pack(32, 15).unwrap(); // row 0
        p.pack(32, 15).unwrap(); // row 1 at y=16
                                 // Next would be at y=32, which is out of bounds
        assert!(p.pack(32, 2).is_none());
    }

    #[test]
    fn test_mixed_heights_shelf() {
        let mut p = ShelfPacker::new(128, 128, 1);
        p.pack(30, 10).unwrap();
        p.pack(30, 20).unwrap(); // taller, shelf height = 20
                                 // Next shelf starts at y = 20 + 1 = 21
        let c = p.pack(128, 10).unwrap();
        assert_eq!(c.y, 21);
    }
}
