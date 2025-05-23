using System;
using System.IO;
using System.Runtime.InteropServices;
using CsBindgen;

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

public unsafe class GoudGame
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
            fixed (byte* titleBytes = System.Text.Encoding.ASCII.GetBytes(title + "\0"))
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

    public void Terminate()
    {
        unsafe
        {
            NativeMethods.game_terminate(gameInstance);
        }
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
            fixed (byte* texturePathBytes = System.Text.Encoding.ASCII.GetBytes(texturePath + "\0"))
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

    public void Close()
    {
        unsafe
        {
            NativeMethods.game_terminate(gameInstance);
        }
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
            fixed (byte* mapNameBytes = System.Text.Encoding.ASCII.GetBytes(mapName + "\0"))
            fixed (byte* mapPathBytes = System.Text.Encoding.ASCII.GetBytes(mapPath + "\0"))
            {
                fixed (uint* textureIdsPtr = textureIds)
                {
                    return NativeMethods.game_load_tiled_map(
                        gameInstance,
                        mapNameBytes,
                        mapPathBytes,
                        textureIdsPtr
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
            fixed (byte* messageBytes = System.Text.Encoding.ASCII.GetBytes(message + "\0"))
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
}
