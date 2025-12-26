using System;
using System.IO;
using System.Runtime.InteropServices;
using CsBindgen;
using GoudEngine.Assets;
using GoudEngine.Config;
using GoudEngine.Core;
using GoudEngine.Entities;
using GoudEngine.Input;
using GoudEngine.Math;

public enum RendererType
{
    Renderer2D = 0,
    Renderer3D = 1
}

public enum PrimitiveType
{
    Cube = 0,
    Sphere = 1,
    Plane = 2,
    Cylinder = 3
}

public enum LightType
{
    Point = 0,
    Directional = 1,
    Spot = 2
}

public enum GridRenderMode
{
    Blend = 0,
    Overlap = 1
}

public unsafe class GoudGame : IDisposable
{
    private GameSdk* gameInstance;
    private bool _isDisposed;

    public delegate void GameCallback();

    public UpdateResponseData UpdateResponseData { get; private set; }

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

    public GoudGame(
        uint width,
        uint height,
        string title,
        RendererType rendererType = RendererType.Renderer2D,
        uint targetFPS = 60
    )
    {
        unsafe
        {
            fixed (byte* titleBytes = System.Text.Encoding.UTF8.GetBytes(title + "\0"))
            {
                gameInstance = NativeMethods.game_create(
                    width,
                    height,
                    titleBytes,
                    targetFPS,
                    (int)rendererType
                );
            }
        }
    }

    public void Initialize(GameCallback initCallback)
    {
        unsafe
        {
            NativeMethods.game_initialize(gameInstance);
            initCallback?.Invoke();
        }
    }

    public void Start(GameCallback startCallback)
    {
        unsafe
        {
            NativeMethods.game_start(gameInstance);
            startCallback?.Invoke();
        }
    }

    public void Update(GameCallback updateCallback)
    {
        unsafe
        {
            while (!ShouldClose())
            {
                UpdateResponseData = NativeMethods.game_update(gameInstance);
                updateCallback?.Invoke();
            }
        }
    }

    public bool ShouldClose()
    {
        unsafe
        {
            return NativeMethods.game_should_close(gameInstance);
        }
    }

    /// <summary>
    /// Terminates the game and releases native resources.
    /// </summary>
    [Obsolete("Use Dispose() instead. Terminate() is kept for backward compatibility.")]
    public void Terminate()
    {
        Dispose();
    }

    public uint AddSprite(SpriteCreateDto data)
    {
        unsafe
        {
            return NativeMethods.game_add_sprite(gameInstance, data);
        }
    }

    public uint CreateTexture(string texturePath)
    {
        unsafe
        {
            fixed (byte* texturePathBytes = System.Text.Encoding.UTF8.GetBytes(texturePath + "\0"))
            {
                return NativeMethods.game_create_texture(gameInstance, texturePathBytes);
            }
        }
    }

    public void UpdateSprite(SpriteUpdateDto data)
    {
        unsafe
        {
            NativeMethods.game_update_sprite(gameInstance, data);
        }
    }

    public void RemoveSprite(uint id)
    {
        unsafe
        {
            NativeMethods.game_remove_sprite(gameInstance, id);
        }
    }

    public bool CheckCollision(uint id1, uint id2)
    {
        unsafe
        {
            return NativeMethods.check_collision_between_sprites(gameInstance, id1, id2);
        }
    }

    /// <summary>
    /// Closes the game window and releases native resources.
    /// </summary>
    [Obsolete("Use Dispose() instead. Close() is kept for backward compatibility.")]
    public void Close()
    {
        Dispose();
    }

    public bool IsKeyPressed(int keyCode)
    {
        unsafe
        {
            return NativeMethods.game_is_key_pressed(gameInstance, keyCode);
        }
    }

    public bool IsMouseButtonPressed(int button)
    {
        unsafe
        {
            return NativeMethods.game_is_mouse_button_pressed(gameInstance, button);
        }
    }

    public MousePosition GetMousePosition()
    {
        unsafe
        {
            return NativeMethods.game_get_mouse_position(gameInstance);
        }
    }

    public uint LoadTiledMap(string mapName, string mapPath, uint[] textureIds)
    {
        unsafe
        {
            fixed (byte* mapNameBytes = System.Text.Encoding.UTF8.GetBytes(mapName + "\0"))
            fixed (byte* mapPathBytes = System.Text.Encoding.UTF8.GetBytes(mapPath + "\0"))
            {
                fixed (uint* textureIdsPtr = textureIds)
                {
                    return NativeMethods.game_load_tiled_map(
                        gameInstance,
                        mapNameBytes,
                        mapPathBytes,
                        textureIdsPtr,
                        (nuint)textureIds.Length
                    );
                }
            }
        }
    }

    public void SetSelectedTiledMapById(uint id)
    {
        unsafe
        {
            NativeMethods.game_set_selected_map_by_id(gameInstance, id);
        }
    }

    public void ClearSelectedTiledMap()
    {
        unsafe
        {
            NativeMethods.game_clear_selected_map(gameInstance);
        }
    }

    public void SetCameraPosition(float x, float y)
    {
        unsafe
        {
            NativeMethods.game_set_camera_position(gameInstance, x, y);
        }
    }

    public void SetCameraPosition3D(float x, float y, float z)
    {
        unsafe
        {
            NativeMethods.game_set_camera_position_3d(gameInstance, x, y, z);
        }
    }

    public float[] GetCameraPosition()
    {
        unsafe
        {
            float[] position = new float[3];
            fixed (float* positionPtr = position)
            {
                NativeMethods.game_get_camera_position(gameInstance, positionPtr);
            }
            return position;
        }
    }

    public void SetCameraRotation(float pitch, float yaw, float roll)
    {
        unsafe
        {
            NativeMethods.game_set_camera_rotation(gameInstance, pitch, yaw, roll);
        }
    }

    public float[] GetCameraRotation()
    {
        unsafe
        {
            float[] rotation = new float[3];
            fixed (float* rotationPtr = rotation)
            {
                NativeMethods.game_get_camera_rotation(gameInstance, rotationPtr);
            }
            return rotation;
        }
    }

    public void SetCameraZoom(float zoom)
    {
        unsafe
        {
            NativeMethods.game_set_camera_zoom(gameInstance, zoom);
        }
    }

    public float GetCameraZoom()
    {
        unsafe
        {
            return NativeMethods.game_get_camera_zoom(gameInstance);
        }
    }

    public void GameLog(string message)
    {
        unsafe
        {
            fixed (byte* messageBytes = System.Text.Encoding.UTF8.GetBytes(message + "\0"))
            {
                NativeMethods.game_log(gameInstance, messageBytes);
            }
        }
    }

    public uint CreatePrimitive(CsBindgen.PrimitiveCreateInfo createInfo)
    {
        return NativeMethods.game_create_primitive(gameInstance, createInfo);
    }

    public bool SetObjectPosition(uint objectId, float x, float y, float z)
    {
        unsafe
        {
            return NativeMethods.game_set_object_position(gameInstance, objectId, x, y, z);
        }
    }

    public bool SetObjectRotation(uint objectId, float x, float y, float z)
    {
        unsafe
        {
            return NativeMethods.game_set_object_rotation(gameInstance, objectId, x, y, z);
        }
    }

    public bool SetObjectScale(uint objectId, float x, float y, float z)
    {
        unsafe
        {
            return NativeMethods.game_set_object_scale(gameInstance, objectId, x, y, z);
        }
    }

    // Convenience methods for common primitives
    public uint CreateCube(
        uint textureId,
        float width = 1.0f,
        float height = 1.0f,
        float depth = 1.0f
    )
    {
        var createInfo = new CsBindgen.PrimitiveCreateInfo
        {
            primitive_type = (CsBindgen.PrimitiveType)PrimitiveType.Cube,
            width = width,
            height = height,
            depth = depth,
            segments = 1,
            texture_id = textureId
        };
        return CreatePrimitive(createInfo);
    }

    public uint CreateSphere(uint textureId, float radius = 1.0f, uint segments = 32)
    {
        var createInfo = new CsBindgen.PrimitiveCreateInfo
        {
            primitive_type = (CsBindgen.PrimitiveType)PrimitiveType.Sphere,
            width = radius * 2,
            height = radius * 2,
            depth = radius * 2,
            segments = segments,
            texture_id = textureId
        };
        return CreatePrimitive(createInfo);
    }

    public uint CreatePlane(uint textureId, float width = 1.0f, float depth = 1.0f)
    {
        var createInfo = new CsBindgen.PrimitiveCreateInfo
        {
            primitive_type = (CsBindgen.PrimitiveType)PrimitiveType.Plane,
            width = width,
            height = 0.0f,
            depth = depth,
            segments = 1,
            texture_id = textureId
        };
        return CreatePrimitive(createInfo);
    }

    public uint CreateCylinder(
        uint textureId,
        float radius = 0.5f,
        float height = 1.0f,
        uint segments = 32
    )
    {
        var createInfo = new CsBindgen.PrimitiveCreateInfo
        {
            primitive_type = (CsBindgen.PrimitiveType)PrimitiveType.Cylinder,
            width = radius * 2,
            height = height,
            depth = radius * 2,
            segments = segments,
            texture_id = textureId
        };
        return CreatePrimitive(createInfo);
    }

    public uint AddLight(
        LightType lightType,
        float positionX,
        float positionY,
        float positionZ,
        float directionX,
        float directionY,
        float directionZ,
        float colorR,
        float colorG,
        float colorB,
        float intensity = 1.0f,
        float temperature = 6500.0f,
        float range = 10.0f,
        float spotAngle = 45.0f
    )
    {
        unsafe
        {
            return NativeMethods.game_add_light(
                gameInstance,
                (int)lightType,
                positionX,
                positionY,
                positionZ,
                directionX,
                directionY,
                directionZ,
                colorR,
                colorG,
                colorB,
                intensity,
                temperature,
                range,
                spotAngle
            );
        }
    }

    public bool RemoveLight(uint lightId)
    {
        unsafe
        {
            return NativeMethods.game_remove_light(gameInstance, lightId);
        }
    }

    public bool UpdateLight(
        uint lightId,
        LightType lightType,
        float positionX,
        float positionY,
        float positionZ,
        float directionX,
        float directionY,
        float directionZ,
        float colorR,
        float colorG,
        float colorB,
        float intensity = 1.0f,
        float temperature = 6500.0f,
        float range = 10.0f,
        float spotAngle = 45.0f
    )
    {
        unsafe
        {
            return NativeMethods.game_update_light(
                gameInstance,
                lightId,
                (int)lightType,
                positionX,
                positionY,
                positionZ,
                directionX,
                directionY,
                directionZ,
                colorR,
                colorG,
                colorB,
                intensity,
                temperature,
                range,
                spotAngle
            );
        }
    }

    // Helper methods for common light setups
    public uint AddPointLight(
        float positionX,
        float positionY,
        float positionZ,
        float colorR = 1.0f,
        float colorG = 1.0f,
        float colorB = 1.0f,
        float intensity = 1.0f,
        float temperature = 6500.0f,
        float range = 10.0f
    )
    {
        return AddLight(
            LightType.Point,
            positionX,
            positionY,
            positionZ,
            0,
            0,
            0, // Direction doesn't matter for point lights
            colorR,
            colorG,
            colorB,
            intensity,
            temperature,
            range,
            0 // Spot angle doesn't matter for point lights
        );
    }

    public uint AddDirectionalLight(
        float directionX,
        float directionY,
        float directionZ,
        float colorR = 1.0f,
        float colorG = 1.0f,
        float colorB = 1.0f,
        float intensity = 1.0f,
        float temperature = 6500.0f
    )
    {
        return AddLight(
            LightType.Directional,
            0,
            0,
            0, // Position doesn't matter for directional lights
            directionX,
            directionY,
            directionZ,
            colorR,
            colorG,
            colorB,
            intensity,
            temperature,
            float.MaxValue, // Range doesn't matter for directional lights
            0 // Spot angle doesn't matter for directional lights
        );
    }

    public uint AddSpotLight(
        float positionX,
        float positionY,
        float positionZ,
        float directionX,
        float directionY,
        float directionZ,
        float spotAngle = 45.0f,
        float colorR = 1.0f,
        float colorG = 1.0f,
        float colorB = 1.0f,
        float intensity = 1.0f,
        float temperature = 6500.0f,
        float range = 10.0f
    )
    {
        return AddLight(
            LightType.Spot,
            positionX,
            positionY,
            positionZ,
            directionX,
            directionY,
            directionZ,
            colorR,
            colorG,
            colorB,
            intensity,
            temperature,
            range,
            spotAngle
        );
    }

    // Grid Configuration Methods

    /// <summary>
    /// Configures the 3D grid with all available options
    /// </summary>
    public bool ConfigureGrid(
        bool enabled = true,
        float size = 20.0f,
        uint divisions = 20,
        float xzColorR = 0.7f,
        float xzColorG = 0.7f,
        float xzColorB = 0.7f,
        float xyColorR = 0.8f,
        float xyColorG = 0.6f,
        float xyColorB = 0.6f,
        float yzColorR = 0.6f,
        float yzColorG = 0.6f,
        float yzColorB = 0.8f,
        float xAxisColorR = 0.9f,
        float xAxisColorG = 0.2f,
        float xAxisColorB = 0.2f,
        float yAxisColorR = 0.2f,
        float yAxisColorG = 0.9f,
        float yAxisColorB = 0.2f,
        float zAxisColorR = 0.2f,
        float zAxisColorG = 0.2f,
        float zAxisColorB = 0.9f,
        float lineWidth = 1.5f,
        float axisLineWidth = 2.5f,
        bool showAxes = true,
        bool showXZPlane = true,
        bool showXYPlane = true,
        bool showYZPlane = true,
        GridRenderMode renderMode = GridRenderMode.Overlap
    )
    {
        unsafe
        {
            return NativeMethods.game_configure_grid(
                gameInstance,
                enabled,
                size,
                divisions,
                xzColorR,
                xzColorG,
                xzColorB,
                xyColorR,
                xyColorG,
                xyColorB,
                yzColorR,
                yzColorG,
                yzColorB,
                xAxisColorR,
                xAxisColorG,
                xAxisColorB,
                yAxisColorR,
                yAxisColorG,
                yAxisColorB,
                zAxisColorR,
                zAxisColorG,
                zAxisColorB,
                lineWidth,
                axisLineWidth,
                showAxes,
                showXZPlane,
                showXYPlane,
                showYZPlane,
                (int)renderMode
            );
        }
    }

    /// <summary>
    /// Simple helper to enable/disable the grid
    /// </summary>
    public bool SetGridEnabled(bool enabled)
    {
        unsafe
        {
            return NativeMethods.game_set_grid_enabled(gameInstance, enabled);
        }
    }

    /// <summary>
    /// Set the grid render mode (blend or overlap)
    /// </summary>
    /// <param name="blendMode">True for Blend mode (grid rendered with proper depth testing),
    /// False for Overlap mode (grid always on top)</param>
    /// <returns>True if successful</returns>
    public bool SetGridRenderMode(bool blendMode)
    {
        unsafe
        {
            return NativeMethods.game_set_grid_render_mode(gameInstance, blendMode);
        }
    }

    /// <summary>
    /// Simple helper to show/hide specific grid planes
    /// </summary>
    public bool SetGridPlanes(bool showXZ, bool showXY, bool showYZ)
    {
        unsafe
        {
            return NativeMethods.game_set_grid_planes(gameInstance, showXZ, showXY, showYZ);
        }
    }

    /// <summary>
    /// Configures the skybox with all available options
    /// </summary>
    public bool ConfigureSkybox(
        bool enabled = true,
        float size = 100.0f,
        uint textureSize = 128,
        float[] rightFaceColor = null,
        float[] leftFaceColor = null,
        float[] topFaceColor = null,
        float[] bottomFaceColor = null,
        float[] frontFaceColor = null,
        float[] backFaceColor = null,
        float blendFactor = 0.5f,
        float[] minColor = null,
        bool useCustomTextures = false
    )
    {
        unsafe
        {
            // Use default colors if not provided
            rightFaceColor = rightFaceColor ?? new float[] { 0.7f, 0.8f, 0.9f };
            leftFaceColor = leftFaceColor ?? new float[] { 0.7f, 0.8f, 0.9f };
            topFaceColor = topFaceColor ?? new float[] { 0.6f, 0.7f, 0.9f };
            bottomFaceColor = bottomFaceColor ?? new float[] { 0.3f, 0.3f, 0.4f };
            frontFaceColor = frontFaceColor ?? new float[] { 0.7f, 0.8f, 0.9f };
            backFaceColor = backFaceColor ?? new float[] { 0.7f, 0.8f, 0.9f };
            minColor = minColor ?? new float[] { 0.1f, 0.1f, 0.2f };

            // The method is already defined in NativeMethods.g.cs so we remove the duplicate implementation
            // and just use the correct call
            return NativeMethods.game_configure_skybox(
                gameInstance,
                enabled,
                size,
                textureSize,
                rightFaceColor[0],
                rightFaceColor[1],
                rightFaceColor[2],
                leftFaceColor[0],
                leftFaceColor[1],
                leftFaceColor[2],
                topFaceColor[0],
                topFaceColor[1],
                topFaceColor[2],
                bottomFaceColor[0],
                bottomFaceColor[1],
                bottomFaceColor[2],
                frontFaceColor[0],
                frontFaceColor[1],
                frontFaceColor[2],
                backFaceColor[0],
                backFaceColor[1],
                backFaceColor[2],
                blendFactor,
                minColor[0],
                minColor[1],
                minColor[2],
                useCustomTextures
            );
        }
    }

    /// <summary>
    /// Simple helper to enable/disable the skybox
    /// </summary>
    public bool SetSkyboxEnabled(bool enabled)
    {
        unsafe
        {
            return NativeMethods.game_set_skybox_enabled(gameInstance, enabled);
        }
    }

    /// <summary>
    /// Set the skybox colors for all faces at once
    /// </summary>
    public bool SetSkyboxColors(
        float[] rightFaceColor,
        float[] leftFaceColor,
        float[] topFaceColor,
        float[] bottomFaceColor,
        float[] frontFaceColor,
        float[] backFaceColor
    )
    {
        unsafe
        {
            return NativeMethods.game_set_skybox_colors(
                gameInstance,
                rightFaceColor[0],
                rightFaceColor[1],
                rightFaceColor[2],
                leftFaceColor[0],
                leftFaceColor[1],
                leftFaceColor[2],
                topFaceColor[0],
                topFaceColor[1],
                topFaceColor[2],
                bottomFaceColor[0],
                bottomFaceColor[1],
                bottomFaceColor[2],
                frontFaceColor[0],
                frontFaceColor[1],
                frontFaceColor[2],
                backFaceColor[0],
                backFaceColor[1],
                backFaceColor[2]
            );
        }
    }

    // IDisposable Implementation

    /// <summary>
    /// Finalizer to ensure native resources are freed if Dispose is not called.
    /// </summary>
    ~GoudGame()
    {
        Dispose(disposing: false);
    }

    /// <summary>
    /// Releases all resources used by the GoudGame.
    /// </summary>
    public void Dispose()
    {
        Dispose(disposing: true);
        GC.SuppressFinalize(this);
    }

    /// <summary>
    /// Releases the unmanaged resources used by GoudGame and optionally releases managed resources.
    /// </summary>
    /// <param name="disposing">true to release both managed and unmanaged resources; false to release only unmanaged resources.</param>
    protected virtual void Dispose(bool disposing)
    {
        if (_isDisposed)
            return;

        // Free unmanaged resources (native game instance)
        if (gameInstance != null)
        {
            NativeMethods.game_terminate(gameInstance);
            gameInstance = null;
        }

        _isDisposed = true;
    }

    /// <summary>
    /// Throws an ObjectDisposedException if this instance has been disposed.
    /// </summary>
    private void ThrowIfDisposed()
    {
        if (_isDisposed)
            throw new ObjectDisposedException(nameof(GoudGame));
    }

    // ========================================
    // New Type-Safe API Methods
    // ========================================

    #region Input with Enums

    /// <summary>
    /// Checks if the specified key is currently pressed.
    /// </summary>
    public bool IsKeyPressed(Keys key)
    {
        return IsKeyPressed((int)key);
    }

    /// <summary>
    /// Checks if the specified mouse button is currently pressed.
    /// </summary>
    public bool IsMouseButtonPressed(MouseButtons button)
    {
        return IsMouseButtonPressed((int)button);
    }

    #endregion

    #region Sprite Methods with New Types

    /// <summary>
    /// Creates a sprite using the configuration builder.
    /// </summary>
    public SpriteId AddSprite(SpriteConfig config)
    {
        var id = AddSprite(config.ToNative());
        return new SpriteId(id);
    }

    /// <summary>
    /// Creates a sprite and returns a high-level wrapper.
    /// </summary>
    public Sprite CreateSprite(SpriteConfig config)
    {
        var id = AddSprite(config);
        return new Sprite(
            this,
            id,
            config.TextureId,
            config.Position,
            config.Dimensions,
            config.Scale,
            config.Rotation,
            config.ZLayer,
            config.SourceRect,
            config.Frame,
            config.Debug
        );
    }

    /// <summary>
    /// Removes a sprite by its type-safe ID.
    /// </summary>
    public void RemoveSprite(SpriteId id)
    {
        RemoveSprite(id.Value);
    }

    /// <summary>
    /// Checks collision between two sprites using type-safe IDs.
    /// </summary>
    public bool CheckCollision(SpriteId id1, SpriteId id2)
    {
        return CheckCollision(id1.Value, id2.Value);
    }

    #endregion

    #region Texture Methods with New Types

    /// <summary>
    /// Creates a texture and returns a type-safe ID.
    /// </summary>
    public TextureId LoadTexture(string texturePath)
    {
        return new TextureId(CreateTexture(texturePath));
    }

    #endregion

    #region Camera Methods with Vectors

    /// <summary>
    /// Sets the 2D camera position using a Vector2.
    /// </summary>
    public void SetCameraPosition(Vector2 position)
    {
        SetCameraPosition(position.X, position.Y);
    }

    /// <summary>
    /// Sets the 3D camera position using a Vector3.
    /// </summary>
    public void SetCameraPosition(Vector3 position)
    {
        SetCameraPosition3D(position.X, position.Y, position.Z);
    }

    /// <summary>
    /// Gets the camera position as a Vector3.
    /// </summary>
    public Vector3 GetCameraPositionVector()
    {
        var pos = GetCameraPosition();
        return new Vector3(pos[0], pos[1], pos[2]);
    }

    /// <summary>
    /// Sets the camera rotation using a Vector3 (pitch, yaw, roll).
    /// </summary>
    public void SetCameraRotation(Vector3 rotation)
    {
        SetCameraRotation(rotation.X, rotation.Y, rotation.Z);
    }

    /// <summary>
    /// Gets the camera rotation as a Vector3.
    /// </summary>
    public Vector3 GetCameraRotationVector()
    {
        var rot = GetCameraRotation();
        return new Vector3(rot[0], rot[1], rot[2]);
    }

    #endregion

    #region 3D Object Methods with New Types

    /// <summary>
    /// Creates a 3D object and returns a high-level wrapper.
    /// </summary>
    public Object3D CreateObject(PrimitiveType type, TextureId textureId, Vector3 position = default)
    {
        var createInfo = new CsBindgen.PrimitiveCreateInfo
        {
            primitive_type = (CsBindgen.PrimitiveType)type,
            width = 1.0f,
            height = 1.0f,
            depth = 1.0f,
            segments = type == PrimitiveType.Sphere ? 32u : 1u,
            texture_id = textureId
        };
        var id = new ObjectId(CreatePrimitive(createInfo));

        if (position != default)
        {
            SetObjectPosition(id, position.X, position.Y, position.Z);
        }

        return new Object3D(this, id, type, textureId, position, Vector3.Zero, Vector3.One);
    }

    /// <summary>
    /// Sets the position of a 3D object using Vector3.
    /// </summary>
    public bool SetObjectPosition(ObjectId objectId, Vector3 position)
    {
        return SetObjectPosition(objectId, position.X, position.Y, position.Z);
    }

    /// <summary>
    /// Sets the rotation of a 3D object using Vector3.
    /// </summary>
    public bool SetObjectRotation(ObjectId objectId, Vector3 rotation)
    {
        return SetObjectRotation(objectId, rotation.X, rotation.Y, rotation.Z);
    }

    /// <summary>
    /// Sets the scale of a 3D object using Vector3.
    /// </summary>
    public bool SetObjectScale(ObjectId objectId, Vector3 scale)
    {
        return SetObjectScale(objectId, scale.X, scale.Y, scale.Z);
    }

    #endregion

    #region Light Methods with New Types

    /// <summary>
    /// Creates a light and returns a high-level wrapper.
    /// </summary>
    public Light CreateLight(
        LightType type,
        Vector3 position,
        Vector3 direction = default,
        Color? color = null,
        float intensity = 1.0f,
        float temperature = 6500.0f,
        float range = 10.0f,
        float spotAngle = 45.0f)
    {
        var lightColor = color ?? Color.White;
        var id = new LightId(AddLight(
            type,
            position.X, position.Y, position.Z,
            direction.X, direction.Y, direction.Z,
            lightColor.R, lightColor.G, lightColor.B,
            intensity,
            temperature,
            range,
            spotAngle
        ));

        return new Light(this, id, type, position, direction, lightColor, intensity, temperature, range, spotAngle);
    }

    /// <summary>
    /// Creates a point light.
    /// </summary>
    public Light CreatePointLight(Vector3 position, Color? color = null, float intensity = 1.0f, float range = 10.0f)
    {
        return CreateLight(LightType.Point, position, Vector3.Zero, color, intensity, range: range);
    }

    /// <summary>
    /// Creates a directional light.
    /// </summary>
    public Light CreateDirectionalLight(Vector3 direction, Color? color = null, float intensity = 1.0f)
    {
        return CreateLight(LightType.Directional, Vector3.Zero, direction, color, intensity);
    }

    /// <summary>
    /// Creates a spot light.
    /// </summary>
    public Light CreateSpotLight(Vector3 position, Vector3 direction, float spotAngle = 45.0f, Color? color = null, float intensity = 1.0f, float range = 10.0f)
    {
        return CreateLight(LightType.Spot, position, direction, color, intensity, range: range, spotAngle: spotAngle);
    }

    /// <summary>
    /// Removes a light by its type-safe ID.
    /// </summary>
    public bool RemoveLight(LightId id)
    {
        return RemoveLight(id.Value);
    }

    #endregion

    #region Grid Configuration with Builder

    /// <summary>
    /// Configures the grid using a GridConfig builder.
    /// </summary>
    public bool ConfigureGrid(GridConfig config)
    {
        return ConfigureGrid(
            config.Enabled,
            config.Size,
            config.Divisions,
            config.XZPlaneColor.R, config.XZPlaneColor.G, config.XZPlaneColor.B,
            config.XYPlaneColor.R, config.XYPlaneColor.G, config.XYPlaneColor.B,
            config.YZPlaneColor.R, config.YZPlaneColor.G, config.YZPlaneColor.B,
            config.XAxisColor.R, config.XAxisColor.G, config.XAxisColor.B,
            config.YAxisColor.R, config.YAxisColor.G, config.YAxisColor.B,
            config.ZAxisColor.R, config.ZAxisColor.G, config.ZAxisColor.B,
            config.LineWidth,
            config.AxisLineWidth,
            config.ShowAxes,
            config.ShowXZPlane,
            config.ShowXYPlane,
            config.ShowYZPlane,
            config.RenderMode
        );
    }

    #endregion

    #region Skybox Configuration with Builder

    /// <summary>
    /// Configures the skybox using a SkyboxConfig builder.
    /// </summary>
    public bool ConfigureSkybox(SkyboxConfig config)
    {
        return ConfigureSkybox(
            config.Enabled,
            config.Size,
            config.TextureSize,
            config.RightFaceColor.ToArray(),
            config.LeftFaceColor.ToArray(),
            config.TopFaceColor.ToArray(),
            config.BottomFaceColor.ToArray(),
            config.FrontFaceColor.ToArray(),
            config.BackFaceColor.ToArray(),
            config.BlendFactor,
            config.MinColor.ToArray(),
            config.UseCustomTextures
        );
    }

    #endregion

    #region Tiled Map Methods with New Types

    /// <summary>
    /// Loads a tiled map and returns a type-safe ID.
    /// </summary>
    public TiledMapId LoadMap(string mapName, string mapPath, TextureId[] textureIds)
    {
        var ids = new uint[textureIds.Length];
        for (int i = 0; i < textureIds.Length; i++)
        {
            ids[i] = textureIds[i].Value;
        }
        return new TiledMapId(LoadTiledMap(mapName, mapPath, ids));
    }

    /// <summary>
    /// Sets the selected tiled map by type-safe ID.
    /// </summary>
    public void SetSelectedMap(TiledMapId id)
    {
        SetSelectedTiledMapById(id.Value);
    }

    #endregion
}
