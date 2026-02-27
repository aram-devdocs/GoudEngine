using System;
using System.IO;
using System.Runtime.InteropServices;
using CsBindgen;
using GoudEngine.Input;
using GoudEngine.Math;

/// <summary>
/// Main game class for immediate-mode rendering.
/// </summary>
public unsafe class GoudGame : IDisposable
{
    private GoudContextId _contextId;
    private bool _isDisposed;
    private float _deltaTime;

    /// <summary>
    /// Gets the context ID for low-level FFI operations.
    /// </summary>
    public GoudContextId ContextId => _contextId;

    /// <summary>
    /// Gets the delta time from the last frame (in seconds).
    /// </summary>
    public float DeltaTime => _deltaTime;

    // Static constructor to initialize the library resolver
    static GoudGame()
    {
        NativeLibrary.SetDllImportResolver(
            typeof(NativeMethods).Assembly,
            (libraryName, assembly, searchPath) =>
            {
                string basePath = AppDomain.CurrentDomain.BaseDirectory;
                string platform = RuntimeInformation.IsOSPlatform(OSPlatform.Windows)
                    ? "win-x64"
                    : RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
                        ? "osx-x64"
                        : "linux-x64";

                string libraryPath = Path.Combine(
                    basePath,
                    "runtimes",
                    platform,
                    "native",
                    $"{libraryName}.dylib"
                );

                if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
                    libraryPath = Path.Combine(
                        basePath,
                        "runtimes",
                        platform,
                        "native",
                        $"{libraryName}.dll"
                    );
                else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
                    libraryPath = Path.Combine(
                        basePath,
                        "runtimes",
                        platform,
                        "native",
                        $"{libraryName}.so"
                    );

                if (NativeLibrary.TryLoad(libraryPath, out IntPtr handle))
                {
                    return handle;
                }

                throw new DllNotFoundException(
                    $"Could not load library '{libraryName}' from path '{libraryPath}'"
                );
            }
        );
    }

    /// <summary>
    /// Creates a new game window with the specified dimensions and title.
    /// </summary>
    public GoudGame(uint width, uint height, string title)
    {
        fixed (byte* titleBytes = System.Text.Encoding.UTF8.GetBytes(title + "\0"))
        {
            _contextId = NativeMethods.goud_window_create(width, height, titleBytes);

            if (_contextId.Item1 == ulong.MaxValue)
            {
                throw new InvalidOperationException("Failed to create game window");
            }

            // Enable blending for proper sprite transparency
            NativeMethods.goud_renderer_enable_blending(_contextId);
        }
    }

    /// <summary>
    /// Returns true if the game window should close.
    /// </summary>
    public bool ShouldClose()
    {
        ThrowIfDisposed();
        return NativeMethods.goud_window_should_close(_contextId);
    }

    /// <summary>
    /// Sets the should close flag to request window closure.
    /// </summary>
    public void Close()
    {
        ThrowIfDisposed();
        NativeMethods.goud_window_set_should_close(_contextId, true);
    }

    /// <summary>
    /// Begins a new frame - polls events and clears the screen.
    /// Call this at the start of your game loop.
    /// </summary>
    public void BeginFrame(float r = 0.1f, float g = 0.1f, float b = 0.1f, float a = 1.0f)
    {
        ThrowIfDisposed();
        _deltaTime = NativeMethods.goud_window_poll_events(_contextId);
        NativeMethods.goud_window_clear(_contextId, r, g, b, a);
        NativeMethods.goud_renderer_begin(_contextId);
        
        // Enable blending for transparency (must be done each frame)
        NativeMethods.goud_renderer_enable_blending(_contextId);
    }

    /// <summary>
    /// Ends the current frame - presents to screen.
    /// Call this at the end of your game loop.
    /// </summary>
    public void EndFrame()
    {
        ThrowIfDisposed();
        NativeMethods.goud_renderer_end(_contextId);
        NativeMethods.goud_window_swap_buffers(_contextId);
    }

    #region Texture Management

    /// <summary>
    /// Loads a texture from the specified file path.
    /// Returns the texture handle (0 on failure).
    /// </summary>
    public ulong LoadTexture(string path)
    {
        ThrowIfDisposed();
        fixed (byte* pathBytes = System.Text.Encoding.UTF8.GetBytes(path + "\0"))
        {
            return NativeMethods.goud_texture_load(_contextId, pathBytes);
        }
    }

    /// <summary>
    /// Alias for LoadTexture for backward compatibility.
    /// </summary>
    public uint CreateTexture(string path)
    {
        return (uint)LoadTexture(path);
    }

    /// <summary>
    /// Destroys a previously loaded texture.
    /// </summary>
    public bool DestroyTexture(ulong texture)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_texture_destroy(_contextId, texture);
    }

    #endregion

    #region Immediate-Mode Rendering

    /// <summary>
    /// Draws a textured sprite at the given position.
    /// This is an immediate-mode draw call - the sprite is rendered immediately
    /// and not retained between frames. Call this each frame in your game loop.
    /// </summary>
    /// <param name="textureId">Texture ID from LoadTexture()</param>
    /// <param name="x">X position</param>
    /// <param name="y">Y position</param>
    /// <param name="width">Width of the sprite</param>
    /// <param name="height">Height of the sprite</param>
    /// <param name="rotation">Rotation in radians (default 0)</param>
    /// <param name="color">Color tint (default white/no tint)</param>
    public bool DrawSprite(ulong textureId, float x, float y,
                           float width, float height, float rotation = 0f,
                           Color? color = null)
    {
        ThrowIfDisposed();
        var c = color ?? Color.White;
        return NativeMethods.goud_renderer_draw_sprite(
            _contextId, textureId, x, y, width, height, rotation,
            c.R, c.G, c.B, c.A);
    }

    /// <summary>
    /// Draws a textured sprite (uint overload for backward compatibility).
    /// </summary>
    public bool DrawSprite(uint textureId, float x, float y,
                           float width, float height, float rotation = 0f,
                           Color? color = null)
    {
        return DrawSprite((ulong)textureId, x, y, width, height, rotation, color);
    }

    /// <summary>
    /// Draws a textured sprite with a source rectangle for sprite sheet animation.
    /// </summary>
    /// <param name="textureId">Texture ID from LoadTexture()</param>
    /// <param name="x">X position</param>
    /// <param name="y">Y position</param>
    /// <param name="width">Width of the sprite on screen</param>
    /// <param name="height">Height of the sprite on screen</param>
    /// <param name="sourceRect">Source rectangle in normalized UV coordinates (0-1)</param>
    /// <param name="rotation">Rotation in radians (default 0)</param>
    /// <param name="color">Color tint (default white/no tint)</param>
    /// <example>
    /// // For a 128x128 sprite sheet with 32x32 frames (4x4 grid):
    /// var frame0 = RectF.FromGrid(0, 0, 4, 4);  // First frame
    /// var frame1 = RectF.FromGrid(1, 0, 4, 4);  // Second frame
    /// game.DrawSprite(textureId, x, y, 64, 64, frame0);
    /// </example>
    public bool DrawSprite(ulong textureId, float x, float y,
                           float width, float height, RectF sourceRect,
                           float rotation = 0f, Color? color = null)
    {
        ThrowIfDisposed();
        var c = color ?? Color.White;
        return NativeMethods.goud_renderer_draw_sprite_rect(
            _contextId, textureId, x, y, width, height, rotation,
            sourceRect.X, sourceRect.Y, sourceRect.Width, sourceRect.Height,
            c.R, c.G, c.B, c.A);
    }

    /// <summary>
    /// Draws a textured sprite with a source rectangle (uint overload).
    /// </summary>
    public bool DrawSprite(uint textureId, float x, float y,
                           float width, float height, RectF sourceRect,
                           float rotation = 0f, Color? color = null)
    {
        return DrawSprite((ulong)textureId, x, y, width, height, sourceRect, rotation, color);
    }

    /// <summary>
    /// Draws a colored quad (no texture) at the given position.
    /// </summary>
    public bool DrawQuad(float x, float y, float width, float height,
                         Color? color = null)
    {
        ThrowIfDisposed();
        var c = color ?? Color.White;
        return NativeMethods.goud_renderer_draw_quad(
            _contextId, x, y, width, height,
            c.R, c.G, c.B, c.A);
    }

    #endregion

    #region Input

    /// <summary>
    /// Checks if the specified key is currently pressed.
    /// </summary>
    public bool IsKeyPressed(Keys key)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_input_key_pressed(_contextId, (int)key);
    }

    /// <summary>
    /// Checks if the specified key is currently pressed (int overload).
    /// </summary>
    public bool IsKeyPressed(int keyCode)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_input_key_pressed(_contextId, keyCode);
    }

    /// <summary>
    /// Checks if the specified key was just pressed this frame.
    /// </summary>
    public bool IsKeyJustPressed(Keys key)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_input_key_just_pressed(_contextId, (int)key);
    }

    /// <summary>
    /// Checks if the specified key was just released this frame.
    /// </summary>
    public bool IsKeyJustReleased(Keys key)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_input_key_just_released(_contextId, (int)key);
    }

    /// <summary>
    /// Checks if the specified mouse button is currently pressed.
    /// </summary>
    public bool IsMouseButtonPressed(MouseButtons button)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_input_mouse_button_pressed(_contextId, (int)button);
    }

    /// <summary>
    /// Checks if the specified mouse button is currently pressed (int overload).
    /// </summary>
    public bool IsMouseButtonPressed(int button)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_input_mouse_button_pressed(_contextId, button);
    }

    /// <summary>
    /// Checks if the specified mouse button was just pressed this frame.
    /// </summary>
    public bool IsMouseButtonJustPressed(MouseButtons button)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_input_mouse_button_just_pressed(_contextId, (int)button);
    }

    /// <summary>
    /// Gets the current mouse position.
    /// </summary>
    public (float X, float Y) GetMousePosition()
    {
        ThrowIfDisposed();
        float x, y;
        NativeMethods.goud_input_get_mouse_position(_contextId, &x, &y);
        return (x, y);
    }

    /// <summary>
    /// Gets the mouse movement delta since last frame.
    /// </summary>
    public (float X, float Y) GetMouseDelta()
    {
        ThrowIfDisposed();
        float dx, dy;
        NativeMethods.goud_input_get_mouse_delta(_contextId, &dx, &dy);
        return (dx, dy);
    }

    /// <summary>
    /// Gets the scroll delta since last frame.
    /// </summary>
    public (float X, float Y) GetScrollDelta()
    {
        ThrowIfDisposed();
        float dx, dy;
        NativeMethods.goud_input_get_scroll_delta(_contextId, &dx, &dy);
        return (dx, dy);
    }

    #endregion

    #region Logging

    /// <summary>
    /// Logs a message (for debugging).
    /// </summary>
    public void GameLog(string message)
    {
        Console.WriteLine($"[GoudEngine] {message}");
    }

    #endregion

    #region 3D Rendering

    /// <summary>
    /// Invalid object handle constant.
    /// </summary>
    public const uint InvalidObject = uint.MaxValue;

    /// <summary>
    /// Invalid light handle constant.
    /// </summary>
    public const uint InvalidLight = uint.MaxValue;

    /// <summary>
    /// Light types for 3D rendering.
    /// </summary>
    public enum LightType
    {
        /// <summary>Point light that emits in all directions.</summary>
        Point = 0,
        /// <summary>Directional light (like the sun).</summary>
        Directional = 1,
        /// <summary>Spot light with a cone.</summary>
        Spot = 2
    }

    // -------------------------------------------------------------------------
    // Primitive Creation
    // -------------------------------------------------------------------------

    /// <summary>
    /// Creates a 3D cube object.
    /// </summary>
    /// <param name="textureId">Texture handle (0 for no texture)</param>
    /// <param name="width">Cube width</param>
    /// <param name="height">Cube height</param>
    /// <param name="depth">Cube depth</param>
    /// <returns>Object ID on success, InvalidObject on failure</returns>
    public uint CreateCube(uint textureId, float width, float height, float depth)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_create_cube(_contextId, textureId, width, height, depth);
    }

    /// <summary>
    /// Creates a 3D plane (flat surface) object.
    /// </summary>
    /// <param name="textureId">Texture handle (0 for no texture)</param>
    /// <param name="width">Plane width (X axis)</param>
    /// <param name="depth">Plane depth (Z axis)</param>
    /// <returns>Object ID on success, InvalidObject on failure</returns>
    public uint CreatePlane(uint textureId, float width, float depth)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_create_plane(_contextId, textureId, width, depth);
    }

    /// <summary>
    /// Creates a 3D sphere object.
    /// </summary>
    /// <param name="textureId">Texture handle (0 for no texture)</param>
    /// <param name="diameter">Sphere diameter</param>
    /// <param name="segments">Number of segments (higher = smoother)</param>
    /// <returns>Object ID on success, InvalidObject on failure</returns>
    public uint CreateSphere(uint textureId, float diameter, uint segments = 16)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_create_sphere(_contextId, textureId, diameter, segments);
    }

    /// <summary>
    /// Creates a 3D cylinder object.
    /// </summary>
    /// <param name="textureId">Texture handle (0 for no texture)</param>
    /// <param name="radius">Cylinder radius</param>
    /// <param name="height">Cylinder height</param>
    /// <param name="segments">Number of segments (higher = smoother)</param>
    /// <returns>Object ID on success, InvalidObject on failure</returns>
    public uint CreateCylinder(uint textureId, float radius, float height, uint segments = 16)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_create_cylinder(_contextId, textureId, radius, height, segments);
    }

    // -------------------------------------------------------------------------
    // Object Manipulation
    // -------------------------------------------------------------------------

    /// <summary>
    /// Sets the position of a 3D object.
    /// </summary>
    public bool SetObjectPosition(uint objectId, float x, float y, float z)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_set_object_position(_contextId, objectId, x, y, z);
    }

    /// <summary>
    /// Sets the rotation of a 3D object (in degrees).
    /// </summary>
    public bool SetObjectRotation(uint objectId, float x, float y, float z)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_set_object_rotation(_contextId, objectId, x, y, z);
    }

    /// <summary>
    /// Sets the scale of a 3D object.
    /// </summary>
    public bool SetObjectScale(uint objectId, float x, float y, float z)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_set_object_scale(_contextId, objectId, x, y, z);
    }

    /// <summary>
    /// Destroys a 3D object.
    /// </summary>
    public bool DestroyObject(uint objectId)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_destroy_object(_contextId, objectId);
    }

    // -------------------------------------------------------------------------
    // Lighting
    // -------------------------------------------------------------------------

    /// <summary>
    /// Adds a light to the 3D scene.
    /// </summary>
    /// <param name="type">Light type</param>
    /// <param name="positionX">Light X position</param>
    /// <param name="positionY">Light Y position</param>
    /// <param name="positionZ">Light Z position</param>
    /// <param name="directionX">Light X direction</param>
    /// <param name="directionY">Light Y direction</param>
    /// <param name="directionZ">Light Z direction</param>
    /// <param name="colorR">Light red component (0-1)</param>
    /// <param name="colorG">Light green component (0-1)</param>
    /// <param name="colorB">Light blue component (0-1)</param>
    /// <param name="intensity">Light intensity</param>
    /// <param name="range">Light range</param>
    /// <param name="spotAngle">Spot cone angle in degrees</param>
    /// <returns>Light ID on success, InvalidLight on failure</returns>
    public uint AddLight(
        LightType type,
        float positionX, float positionY, float positionZ,
        float directionX, float directionY, float directionZ,
        float colorR, float colorG, float colorB,
        float intensity = 1.0f,
        float range = 10.0f,
        float spotAngle = 45.0f)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_add_light(
            _contextId, (int)type,
            positionX, positionY, positionZ,
            directionX, directionY, directionZ,
            colorR, colorG, colorB,
            intensity, range, spotAngle);
    }

    /// <summary>
    /// Adds a point light to the scene (convenience method).
    /// </summary>
    public uint AddPointLight(float x, float y, float z, float r, float g, float b, 
                              float intensity = 1.0f, float range = 10.0f)
    {
        return AddLight(LightType.Point, x, y, z, 0, -1, 0, r, g, b, intensity, range, 0);
    }

    /// <summary>
    /// Updates a light's properties.
    /// </summary>
    public bool UpdateLight(
        uint lightId,
        LightType type,
        float positionX, float positionY, float positionZ,
        float directionX, float directionY, float directionZ,
        float colorR, float colorG, float colorB,
        float intensity = 1.0f,
        float range = 10.0f,
        float spotAngle = 45.0f)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_update_light(
            _contextId, lightId, (int)type,
            positionX, positionY, positionZ,
            directionX, directionY, directionZ,
            colorR, colorG, colorB,
            intensity, range, spotAngle);
    }

    /// <summary>
    /// Removes a light from the scene.
    /// </summary>
    public bool RemoveLight(uint lightId)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_remove_light(_contextId, lightId);
    }

    // -------------------------------------------------------------------------
    // Camera
    // -------------------------------------------------------------------------

    /// <summary>
    /// Sets the 3D camera position.
    /// </summary>
    public bool SetCameraPosition3D(float x, float y, float z)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_set_camera_position(_contextId, x, y, z);
    }

    /// <summary>
    /// Sets the 3D camera rotation (pitch, yaw, roll in degrees).
    /// </summary>
    public bool SetCameraRotation(float pitch, float yaw, float roll = 0f)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_set_camera_rotation(_contextId, pitch, yaw, roll);
    }

    // -------------------------------------------------------------------------
    // Grid and Skybox
    // -------------------------------------------------------------------------

    /// <summary>
    /// Configures the ground grid.
    /// </summary>
    /// <param name="enabled">Whether grid is visible</param>
    /// <param name="size">Grid size</param>
    /// <param name="divisions">Number of grid divisions</param>
    public bool ConfigureGrid(bool enabled, float size = 20f, uint divisions = 20)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_configure_grid(_contextId, enabled, size, divisions);
    }

    /// <summary>
    /// Sets grid visibility.
    /// </summary>
    public bool SetGridEnabled(bool enabled)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_set_grid_enabled(_contextId, enabled);
    }

    /// <summary>
    /// Configures the skybox/background color.
    /// </summary>
    public bool ConfigureSkybox(bool enabled, float r = 0.1f, float g = 0.1f, float b = 0.2f, float a = 1.0f)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_configure_skybox(_contextId, enabled, r, g, b, a);
    }

    /// <summary>
    /// Configures fog settings for atmospheric effects.
    /// </summary>
    /// <param name="enabled">Whether fog is enabled</param>
    /// <param name="r">Fog color red component (0-1)</param>
    /// <param name="g">Fog color green component (0-1)</param>
    /// <param name="b">Fog color blue component (0-1)</param>
    /// <param name="density">Fog density (higher = thicker fog)</param>
    public bool ConfigureFog(bool enabled, float r = 0.05f, float g = 0.05f, float b = 0.1f, float density = 0.02f)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_configure_fog(_contextId, enabled, r, g, b, density);
    }

    /// <summary>
    /// Sets fog enabled state.
    /// </summary>
    public bool SetFogEnabled(bool enabled)
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_set_fog_enabled(_contextId, enabled);
    }

    // -------------------------------------------------------------------------
    // Rendering
    // -------------------------------------------------------------------------

    /// <summary>
    /// Renders all 3D objects in the scene.
    /// Call this between BeginFrame and EndFrame.
    /// </summary>
    public bool Render3D()
    {
        ThrowIfDisposed();
        return NativeMethods.goud_renderer3d_render_all(_contextId);
    }

    #endregion

    #region IDisposable

    ~GoudGame()
    {
        Dispose(disposing: false);
    }

    public void Dispose()
    {
        Dispose(disposing: true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (_isDisposed)
            return;

        if (_contextId.Item1 != ulong.MaxValue)
        {
            NativeMethods.goud_window_destroy(_contextId);
            _contextId = new GoudContextId { Item1 = ulong.MaxValue };
        }

        _isDisposed = true;
    }

    private void ThrowIfDisposed()
    {
        if (_isDisposed)
            throw new ObjectDisposedException(nameof(GoudGame));
    }

    #endregion

    #region Legacy API (deprecated)

    /// <summary>
    /// Terminates the game and releases native resources.
    /// </summary>
    [Obsolete("Use Dispose() instead.")]
    public void Terminate()
    {
        Dispose();
    }

    #endregion
}
