using GoudEngine.Math;

namespace GoudEngine.Config;

/// <summary>
/// Configuration for the 3D grid.
/// </summary>
public class GridConfig
{
    /// <summary>
    /// Whether the grid is enabled.
    /// </summary>
    public bool Enabled { get; set; } = true;

    /// <summary>
    /// The size of the grid.
    /// </summary>
    public float Size { get; set; } = 20.0f;

    /// <summary>
    /// The number of divisions in the grid.
    /// </summary>
    public uint Divisions { get; set; } = 20;

    /// <summary>
    /// Color of the XZ plane grid.
    /// </summary>
    public Color XZPlaneColor { get; set; } = new(0.7f, 0.7f, 0.7f);

    /// <summary>
    /// Color of the XY plane grid.
    /// </summary>
    public Color XYPlaneColor { get; set; } = new(0.8f, 0.6f, 0.6f);

    /// <summary>
    /// Color of the YZ plane grid.
    /// </summary>
    public Color YZPlaneColor { get; set; } = new(0.6f, 0.6f, 0.8f);

    /// <summary>
    /// Color of the X axis.
    /// </summary>
    public Color XAxisColor { get; set; } = new(0.9f, 0.2f, 0.2f);

    /// <summary>
    /// Color of the Y axis.
    /// </summary>
    public Color YAxisColor { get; set; } = new(0.2f, 0.9f, 0.2f);

    /// <summary>
    /// Color of the Z axis.
    /// </summary>
    public Color ZAxisColor { get; set; } = new(0.2f, 0.2f, 0.9f);

    /// <summary>
    /// Width of grid lines.
    /// </summary>
    public float LineWidth { get; set; } = 1.5f;

    /// <summary>
    /// Width of axis lines.
    /// </summary>
    public float AxisLineWidth { get; set; } = 2.5f;

    /// <summary>
    /// Whether to show the axis lines.
    /// </summary>
    public bool ShowAxes { get; set; } = true;

    /// <summary>
    /// Whether to show the XZ plane.
    /// </summary>
    public bool ShowXZPlane { get; set; } = true;

    /// <summary>
    /// Whether to show the XY plane.
    /// </summary>
    public bool ShowXYPlane { get; set; } = true;

    /// <summary>
    /// Whether to show the YZ plane.
    /// </summary>
    public bool ShowYZPlane { get; set; } = true;

    /// <summary>
    /// The render mode for the grid (Blend or Overlap).
    /// </summary>
    public GridRenderMode RenderMode { get; set; } = GridRenderMode.Overlap;

    /// <summary>
    /// Creates a default grid configuration.
    /// </summary>
    public static GridConfig Default => new();

    /// <summary>
    /// Creates a minimal grid with only the XZ plane.
    /// </summary>
    public static GridConfig Minimal => new()
    {
        ShowXYPlane = false,
        ShowYZPlane = false,
        ShowAxes = false
    };

    /// <summary>
    /// Creates a configuration with all planes visible.
    /// </summary>
    public static GridConfig AllPlanes => new()
    {
        ShowXZPlane = true,
        ShowXYPlane = true,
        ShowYZPlane = true
    };
}
