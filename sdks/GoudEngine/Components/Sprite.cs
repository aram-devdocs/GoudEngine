using System;
using System.Runtime.InteropServices;
using GoudEngine.Math;

namespace GoudEngine.Components
{
    /// <summary>
    /// 2D sprite component for rendering textured quads.
    /// </summary>
    /// <remarks>
    /// Sprites are textured rectangles that can be tinted, flipped, and have custom
    /// UV coordinates for sprite sheets.
    ///
    /// Memory layout: 48 bytes total
    /// - texture: AssetHandle (8 bytes)
    /// - color: Color (16 bytes)
    /// - source_rect: Option&lt;Rect&gt; (20 bytes: 4 bool + 16 data)
    /// - flip_x: bool (1 byte)
    /// - flip_y: bool (1 byte)
    /// - anchor: Vec2 (8 bytes)
    /// - custom_size: Option&lt;Vec2&gt; (12 bytes: 4 bool + 8 data)
    ///
    /// Note: Actual size may vary due to alignment/padding.
    /// </remarks>
    [StructLayout(LayoutKind.Sequential)]
    [Component(0xFEDCBA0987654321, 48)] // TODO: Compute actual TypeId hash
    public struct Sprite : IComponent
    {
        /// <summary>
        /// Handle to the texture asset. Value of 0 means no texture (will render as colored quad).
        /// </summary>
        public ulong TextureHandle;

        /// <summary>
        /// RGBA color tint applied to the sprite (multiplied with texture colors).
        /// Default is Color.White (no tinting).
        /// </summary>
        public Color Color;

        /// <summary>
        /// Source rectangle in texture UV coordinates for sprite sheets.
        /// If null, uses the entire texture.
        /// </summary>
        public Rect? SourceRect;

        /// <summary>
        /// If true, flip the sprite horizontally.
        /// </summary>
        [MarshalAs(UnmanagedType.U1)]
        public bool FlipX;

        /// <summary>
        /// If true, flip the sprite vertically.
        /// </summary>
        [MarshalAs(UnmanagedType.U1)]
        public bool FlipY;

        /// <summary>
        /// Anchor point in normalized coordinates (0.0-1.0).
        /// Default is (0.5, 0.5) for center.
        /// (0, 0) = top-left, (1, 1) = bottom-right.
        /// </summary>
        public Vector2 Anchor;

        /// <summary>
        /// Custom size override in world units.
        /// If null, uses texture size.
        /// </summary>
        public Vector2? CustomSize;

        /// <inheritdoc/>
        public ulong TypeId => 0xFEDCBA0987654321; // TODO: Compute actual hash

        /// <inheritdoc/>
        public int SizeInBytes => 48;

        /// <summary>
        /// Creates a new Sprite with the specified texture handle.
        /// </summary>
        /// <param name="textureHandle">Handle to the texture asset (0 for none).</param>
        public Sprite(ulong textureHandle)
        {
            TextureHandle = textureHandle;
            Color = Color.White;
            SourceRect = null;
            FlipX = false;
            FlipY = false;
            Anchor = new Vector2(0.5f, 0.5f); // Center anchor
            CustomSize = null;
        }

        /// <summary>
        /// Creates a new white sprite (no texture).
        /// </summary>
        /// <returns>A white colored sprite.</returns>
        public static Sprite Default()
        {
            return new Sprite(0)
            {
                Color = Color.White
            };
        }

        /// <summary>
        /// Builder: Sets the color tint.
        /// </summary>
        /// <param name="color">The RGBA color.</param>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithColor(Color color)
        {
            Color = color;
            return this;
        }

        /// <summary>
        /// Builder: Sets the source rectangle for sprite sheets.
        /// </summary>
        /// <param name="rect">The UV rectangle in texture coordinates.</param>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithSourceRect(Rect rect)
        {
            SourceRect = rect;
            return this;
        }

        /// <summary>
        /// Builder: Clears the source rectangle (uses full texture).
        /// </summary>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithoutSourceRect()
        {
            SourceRect = null;
            return this;
        }

        /// <summary>
        /// Builder: Sets horizontal flip.
        /// </summary>
        /// <param name="flip">True to flip horizontally.</param>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithFlipX(bool flip = true)
        {
            FlipX = flip;
            return this;
        }

        /// <summary>
        /// Builder: Sets vertical flip.
        /// </summary>
        /// <param name="flip">True to flip vertically.</param>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithFlipY(bool flip = true)
        {
            FlipY = flip;
            return this;
        }

        /// <summary>
        /// Builder: Sets both horizontal and vertical flip.
        /// </summary>
        /// <param name="flipX">True to flip horizontally.</param>
        /// <param name="flipY">True to flip vertically.</param>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithFlip(bool flipX, bool flipY)
        {
            FlipX = flipX;
            FlipY = flipY;
            return this;
        }

        /// <summary>
        /// Builder: Sets the anchor point.
        /// </summary>
        /// <param name="x">Horizontal anchor (0.0-1.0).</param>
        /// <param name="y">Vertical anchor (0.0-1.0).</param>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithAnchor(float x, float y)
        {
            Anchor = new Vector2(x, y);
            return this;
        }

        /// <summary>
        /// Builder: Sets the anchor point.
        /// </summary>
        /// <param name="anchor">The anchor vector.</param>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithAnchorVec(Vector2 anchor)
        {
            Anchor = anchor;
            return this;
        }

        /// <summary>
        /// Builder: Sets a custom size override.
        /// </summary>
        /// <param name="size">The custom size in world units.</param>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithCustomSize(Vector2 size)
        {
            CustomSize = size;
            return this;
        }

        /// <summary>
        /// Builder: Clears the custom size (uses texture size).
        /// </summary>
        /// <returns>This sprite for method chaining.</returns>
        public Sprite WithoutCustomSize()
        {
            CustomSize = null;
            return this;
        }

        /// <summary>
        /// Checks if the sprite has a source rectangle set.
        /// </summary>
        public bool HasSourceRect => SourceRect.HasValue;

        /// <summary>
        /// Checks if the sprite has a custom size set.
        /// </summary>
        public bool HasCustomSize => CustomSize.HasValue;

        /// <summary>
        /// Checks if the sprite is flipped on either axis.
        /// </summary>
        public bool IsFlipped => FlipX || FlipY;

        /// <inheritdoc/>
        public override string ToString()
        {
            var parts = new System.Collections.Generic.List<string>
            {
                $"tex: {TextureHandle}",
                $"color: {Color}"
            };

            if (HasSourceRect)
            {
                parts.Add($"src: {SourceRect}");
            }

            if (FlipX)
            {
                parts.Add("flipX");
            }

            if (FlipY)
            {
                parts.Add("flipY");
            }

            if (HasCustomSize)
            {
                parts.Add($"size: {CustomSize}");
            }

            return $"Sprite({string.Join(", ", parts)})";
        }
    }

    /// <summary>
    /// RGBA color with components in the range [0.0, 1.0].
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct Color
    {
        public float R;
        public float G;
        public float B;
        public float A;

        public Color(float r, float g, float b, float a = 1.0f)
        {
            R = r;
            G = g;
            B = b;
            A = a;
        }

        public static readonly Color White = new Color(1, 1, 1, 1);
        public static readonly Color Black = new Color(0, 0, 0, 1);
        public static readonly Color Red = new Color(1, 0, 0, 1);
        public static readonly Color Green = new Color(0, 1, 0, 1);
        public static readonly Color Blue = new Color(0, 0, 1, 1);
        public static readonly Color Transparent = new Color(0, 0, 0, 0);

        public override string ToString() => $"RGBA({R:F2}, {G:F2}, {B:F2}, {A:F2})";
    }

    /// <summary>
    /// Rectangle with position and size.
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct Rect
    {
        public float X;
        public float Y;
        public float Width;
        public float Height;

        public Rect(float x, float y, float width, float height)
        {
            X = x;
            Y = y;
            Width = width;
            Height = height;
        }

        public override string ToString() => $"Rect({X}, {Y}, {Width}x{Height})";
    }
}
