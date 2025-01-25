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

    public void SetCameraZoom(float zoom)
    {
        unsafe
        {
            NativeMethods.game_set_camera_zoom(gameInstance, zoom);
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
            0, 0, 0,  // Direction doesn't matter for point lights
            colorR,
            colorG,
            colorB,
            intensity,
            temperature,
            range,
            0  // Spot angle doesn't matter for point lights
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
            0, 0, 0,  // Position doesn't matter for directional lights
            directionX,
            directionY,
            directionZ,
            colorR,
            colorG,
            colorB,
            intensity,
            temperature,
            float.MaxValue,  // Range doesn't matter for directional lights
            0  // Spot angle doesn't matter for directional lights
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
}
