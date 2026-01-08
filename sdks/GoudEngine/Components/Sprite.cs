using System;
using System.Runtime.InteropServices;
using GoudEngine.Math;

namespace GoudEngine.Components
{
    /// <summary>
    /// 2D sprite component for rendering textured quads.
    /// </summary>
    /// <remarks>
    /// This is a pure data struct. All sprite manipulation logic is implemented in
    /// the Rust engine and accessible via FFI functions (goud_sprite_*).
    /// 
    /// Sprites are textured rectangles that can be tinted, flipped, and have custom
    /// UV coordinates for sprite sheets.
    /// 
    /// Note: Memory layout must match Rust FfiSprite.
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
        /// Color tint red component (0.0 - 1.0).
        /// </summary>
        public float ColorR;

        /// <summary>
        /// Color tint green component (0.0 - 1.0).
        /// </summary>
        public float ColorG;

        /// <summary>
        /// Color tint blue component (0.0 - 1.0).
        /// </summary>
        public float ColorB;

        /// <summary>
        /// Color tint alpha component (0.0 - 1.0).
        /// </summary>
        public float ColorA;

        /// <summary>
        /// Source rectangle X position (if HasSourceRect is true).
        /// </summary>
        public float SourceRectX;

        /// <summary>
        /// Source rectangle Y position.
        /// </summary>
        public float SourceRectY;

        /// <summary>
        /// Source rectangle width.
        /// </summary>
        public float SourceRectWidth;

        /// <summary>
        /// Source rectangle height.
        /// </summary>
        public float SourceRectHeight;

        /// <summary>
        /// Whether source_rect is set.
        /// </summary>
        [MarshalAs(UnmanagedType.U1)]
        public bool HasSourceRect;

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
        /// Anchor point X in normalized coordinates (0.0-1.0).
        /// </summary>
        public float AnchorX;

        /// <summary>
        /// Anchor point Y in normalized coordinates (0.0-1.0).
        /// </summary>
        public float AnchorY;

        /// <summary>
        /// Custom size width (if HasCustomSize is true).
        /// </summary>
        public float CustomSizeX;

        /// <summary>
        /// Custom size height.
        /// </summary>
        public float CustomSizeY;

        /// <summary>
        /// Whether custom_size is set.
        /// </summary>
        [MarshalAs(UnmanagedType.U1)]
        public bool HasCustomSize;

        /// <inheritdoc/>
        public ulong TypeId => 0xFEDCBA0987654321; // TODO: Compute actual hash

        /// <inheritdoc/>
        public int SizeInBytes => 48;

        /// <summary>
        /// Creates a new Sprite with the specified texture handle and default settings.
        /// </summary>
        /// <param name="textureHandle">Handle to the texture asset (0 for none).</param>
        public Sprite(ulong textureHandle)
        {
            TextureHandle = textureHandle;
            ColorR = 1.0f;
            ColorG = 1.0f;
            ColorB = 1.0f;
            ColorA = 1.0f;
            SourceRectX = 0.0f;
            SourceRectY = 0.0f;
            SourceRectWidth = 0.0f;
            SourceRectHeight = 0.0f;
            HasSourceRect = false;
            FlipX = false;
            FlipY = false;
            AnchorX = 0.5f;
            AnchorY = 0.5f;
            CustomSizeX = 0.0f;
            CustomSizeY = 0.0f;
            HasCustomSize = false;
        }

        /// <summary>
        /// Creates a new white sprite (no texture).
        /// </summary>
        public static Sprite Default() => new(0);

        /// <summary>
        /// Gets or sets the color tint.
        /// </summary>
        public Color Color
        {
            readonly get => new(ColorR, ColorG, ColorB, ColorA);
            set
            {
                ColorR = value.R;
                ColorG = value.G;
                ColorB = value.B;
                ColorA = value.A;
            }
        }

        /// <summary>
        /// Gets or sets the anchor point as a Vector2 (normalized 0.0-1.0).
        /// </summary>
        public Vector2 Anchor
        {
            readonly get => new(AnchorX, AnchorY);
            set
            {
                AnchorX = value.X;
                AnchorY = value.Y;
            }
        }

        /// <summary>
        /// Checks if the sprite is flipped on either axis.
        /// </summary>
        public readonly bool IsFlipped => FlipX || FlipY;

        /// <inheritdoc/>
        public override readonly string ToString()
        {
            var parts = new System.Collections.Generic.List<string>
            {
                $"tex: {TextureHandle}",
                $"color: RGBA({ColorR:F2}, {ColorG:F2}, {ColorB:F2}, {ColorA:F2})"
            };

            if (HasSourceRect)
            {
                parts.Add($"src: ({SourceRectX}, {SourceRectY}, {SourceRectWidth}x{SourceRectHeight})");
            }

            if (FlipX) parts.Add("flipX");
            if (FlipY) parts.Add("flipY");

            if (HasCustomSize)
            {
                parts.Add($"size: ({CustomSizeX}, {CustomSizeY})");
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

        public static readonly Color White = new(1, 1, 1, 1);
        public static readonly Color Black = new(0, 0, 0, 1);
        public static readonly Color Red = new(1, 0, 0, 1);
        public static readonly Color Green = new(0, 1, 0, 1);
        public static readonly Color Blue = new(0, 0, 1, 1);
        public static readonly Color Yellow = new(1, 1, 0, 1);
        public static readonly Color Transparent = new(0, 0, 0, 0);

        public override readonly string ToString() => $"RGBA({R:F2}, {G:F2}, {B:F2}, {A:F2})";
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

        public override readonly string ToString() => $"Rect({X}, {Y}, {Width}x{Height})";
    }
}
