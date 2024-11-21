using System;
using System.IO;
using System.Runtime.InteropServices;
using CsBindgen;

public class GoudGame
{
    private unsafe GameSdk* gameInstance;

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

    public GoudGame(uint width, uint height, string title, uint targetFPS = 60)
    {
        unsafe
        {
            fixed (byte* titleBytes = System.Text.Encoding.ASCII.GetBytes(title + "\0"))
            {
                gameInstance = NativeMethods.game_create(
                    width,
                    height,
                    titleBytes,
                    target_fps: targetFPS
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
}
