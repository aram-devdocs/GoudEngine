// Core/IsometricUtils.cs
// Isometric utility functions

namespace IsometricRpg.Core;

public static class IsometricUtils
{
    public static float Distance(float x1, float y1, float x2, float y2)
    {
        float dx = x2 - x1;
        float dy = y2 - y1;
        return (float)System.Math.Sqrt(dx * dx + dy * dy);
    }

    public static int CalculateZLayer(float y)
    {
        // Higher Y = rendered on top
        return (int)(y / 10);
    }
}
