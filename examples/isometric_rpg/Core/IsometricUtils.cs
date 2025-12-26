namespace IsometricRpg.Core;

/// <summary>
/// Utility class for isometric coordinate conversions and calculations.
/// Uses a "fake" isometric approach where 2D sprites are rendered with
/// proper depth sorting to create the illusion of isometric perspective.
/// </summary>
public static class IsometricUtils
{
    // Isometric tile dimensions (2:1 ratio typical for isometric)
    public const float TileWidth = 64f;
    public const float TileHeight = 32f;

    /// <summary>
    /// 8 directions for isometric movement.
    /// Direction 0 = North (up-right on screen), going clockwise.
    /// </summary>
    public static readonly (float dx, float dy)[] DirectionVectors = new[]
    {
        (0.5f, -0.25f),   // 0: N (up-right)
        (0.5f, 0.0f),     // 1: NE (right)
        (0.5f, 0.25f),    // 2: E (down-right)
        (0.0f, 0.5f),     // 3: SE (down)
        (-0.5f, 0.25f),   // 4: S (down-left)
        (-0.5f, 0.0f),    // 5: SW (left)
        (-0.5f, -0.25f),  // 6: W (up-left)
        (0.0f, -0.5f)     // 7: NW (up)
    };

    /// <summary>
    /// Convert isometric grid coordinates to screen coordinates.
    /// </summary>
    public static (float screenX, float screenY) IsoToScreen(float isoX, float isoY)
    {
        float screenX = (isoX - isoY) * (TileWidth / 2);
        float screenY = (isoX + isoY) * (TileHeight / 2);
        return (screenX, screenY);
    }

    /// <summary>
    /// Convert screen coordinates to isometric grid coordinates.
    /// </summary>
    public static (float isoX, float isoY) ScreenToIso(float screenX, float screenY)
    {
        float isoX = (screenX / (TileWidth / 2) + screenY / (TileHeight / 2)) / 2;
        float isoY = (screenY / (TileHeight / 2) - screenX / (TileWidth / 2)) / 2;
        return (isoX, isoY);
    }

    /// <summary>
    /// Calculate z-layer for depth sorting based on screen Y position.
    /// Higher Y = closer to camera = rendered on top.
    /// </summary>
    public static int CalculateZLayer(float screenY, int baseLayer = 1)
    {
        // Divide by 10 to reduce layer count, add base layer
        return (int)(screenY / 10) + baseLayer;
    }

    /// <summary>
    /// Get direction index (0-7) from movement delta.
    /// Returns -1 if no movement.
    /// </summary>
    public static int GetDirectionFromDelta(float dx, float dy)
    {
        if (MathF.Abs(dx) < 0.001f && MathF.Abs(dy) < 0.001f)
            return -1;

        // Calculate angle in radians, then convert to direction index
        float angle = MathF.Atan2(dy, dx);

        // Convert from radians (-PI to PI) to 0-7 direction
        // Offset by PI/8 (22.5 degrees) to center each direction sector
        float normalized = (angle + MathF.PI) / (MathF.PI * 2); // 0 to 1
        int direction = (int)((normalized * 8 + 0.5f) % 8);

        // Remap to our direction scheme (0=N, going clockwise)
        // Our 0 is up-right, atan2's 0 is right
        direction = (direction + 6) % 8; // Adjust offset

        return direction;
    }

    /// <summary>
    /// Get direction index (0-7) from WASD input.
    /// W=up, S=down, A=left, D=right mapped to isometric directions.
    /// </summary>
    public static int GetDirectionFromInput(float inputX, float inputY)
    {
        if (MathF.Abs(inputX) < 0.001f && MathF.Abs(inputY) < 0.001f)
            return -1;

        // Map WASD input to isometric direction
        // inputX: -1=left(A), +1=right(D)
        // inputY: -1=up(W), +1=down(S)

        if (inputY < 0 && inputX == 0) return 7;      // W only -> NW (up)
        if (inputY < 0 && inputX > 0) return 0;       // W+D -> N (up-right)
        if (inputY == 0 && inputX > 0) return 1;      // D only -> NE (right)
        if (inputY > 0 && inputX > 0) return 2;       // S+D -> E (down-right)
        if (inputY > 0 && inputX == 0) return 3;      // S only -> SE (down)
        if (inputY > 0 && inputX < 0) return 4;       // S+A -> S (down-left)
        if (inputY == 0 && inputX < 0) return 5;      // A only -> SW (left)
        if (inputY < 0 && inputX < 0) return 6;       // W+A -> W (up-left)

        return 3; // Default: facing down
    }

    /// <summary>
    /// Calculate distance between two points.
    /// </summary>
    public static float Distance(float x1, float y1, float x2, float y2)
    {
        float dx = x2 - x1;
        float dy = y2 - y1;
        return MathF.Sqrt(dx * dx + dy * dy);
    }

    /// <summary>
    /// Normalize a vector (make it unit length).
    /// </summary>
    public static (float nx, float ny) Normalize(float dx, float dy)
    {
        float length = MathF.Sqrt(dx * dx + dy * dy);
        if (length < 0.0001f) return (0, 0);
        return (dx / length, dy / length);
    }
}
