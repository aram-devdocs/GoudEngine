using System;
using CsBindgen;

public class GoudGame
{
    private unsafe GameSdk* gameInstance;

    public delegate void GameCallback();

    public GoudGame(uint width, uint height, string title)
    {
        unsafe
        {
            fixed (byte* titleBytes = System.Text.Encoding.ASCII.GetBytes(title + "\0"))
            {
                gameInstance = NativeMethods.game_create(width, height, titleBytes);
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
                NativeMethods.game_update(gameInstance);
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

    public void AddSprite(string texturePath, float x, float y, float scaleX, float scaleY, float rotation)
    {
        unsafe
        {
            fixed (byte* texturePathBytes = System.Text.Encoding.ASCII.GetBytes(texturePath + "\0"))
            {
                NativeMethods.game_add_sprite(gameInstance, texturePathBytes, x, y, scaleX, scaleY, rotation);
            }
        }
    }

    public void UpdateSprite(int index, float x, float y, float scaleX, float scaleY, float rotation)
    {
        unsafe
        {
            NativeMethods.game_update_sprite(gameInstance, (nuint)index, x, y, scaleX, scaleY, rotation);
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