using System;
using CsBindgen;

public class GoudGame
{




    private unsafe GameSdk* gameInstance;

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

    public bool IsKeyPressed(int keyCode)
    {
        unsafe
        {
            return NativeMethods.game_is_key_pressed(gameInstance, keyCode);
        }
    }

    public int AddSprite(string texturePath, SpriteDto data)

    {
        unsafe
        {
            fixed (byte* texturePathBytes = System.Text.Encoding.ASCII.GetBytes(texturePath + "\0"))
            {
                return (int)NativeMethods.game_add_sprite(gameInstance, texturePathBytes, data);
            }
        }
    }

    public void UpdateSprite(int id, SpriteDto data)
    {
        unsafe
        {
            NativeMethods.game_update_sprite(gameInstance, (uint)id, data);
        }
    }

    public void Close()
    {
        unsafe
        {
            NativeMethods.game_terminate(gameInstance);
        }
    }



}