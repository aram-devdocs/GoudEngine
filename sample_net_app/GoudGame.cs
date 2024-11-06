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
            fixed (byte* titleBytes = System.Text.Encoding.ASCII.GetBytes(title))
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

    public void Run(GameCallback updateCallback)
    {
        unsafe
        {
            // while (!NativeMethods.game_should_close(gameInstance))
            // {
            NativeMethods.game_start(gameInstance);
            updateCallback?.Invoke();
            // }
        }
    }

    public void Terminate()
    {
        unsafe
        {
            NativeMethods.game_terminate(gameInstance);
        }
    }

    public void AddSprite(string texturePath, float x, float y, float scaleX, float scaleY, float rotation)
    {
        unsafe
        {
            fixed (byte* texturePathBytes = System.Text.Encoding.ASCII.GetBytes(texturePath))
            {
                NativeMethods.game_add_sprite(gameInstance, texturePathBytes, x, y, scaleX, scaleY, rotation);
            }
        }
    }

    public void Close()
    {
        unsafe
        {
            NativeMethods.game_close_window(gameInstance);
        }
    }
}