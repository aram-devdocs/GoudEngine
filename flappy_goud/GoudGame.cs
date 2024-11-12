using System;
using CsBindgen;

public class GoudGame
{




#pragma warning disable CS8500 // This takes the address of, gets the size of, or declares a pointer to a managed type
    private unsafe GameSdk* gameInstance;
#pragma warning restore CS8500 // This takes the address of, gets the size of, or declares a pointer to a managed type

    public delegate void GameCallback();


    public UpdateResponseData UpdateResponseData { get; private set; }

    public GoudGame(uint width, uint height, string title, uint TargetFPS = 60)
    {
        unsafe
        {
            fixed (byte* titleBytes = System.Text.Encoding.ASCII.GetBytes(title + "\0"))
            {


                gameInstance = NativeMethods.game_create(width, height, titleBytes, target_fps: TargetFPS);
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

    public void UpdateSprite(uint id, SpriteUpdateDto data)
    {
        unsafe
        {
            NativeMethods.game_update_sprite(gameInstance, id, data);
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









}