using System;
using System.Collections.Generic;

public class Movement
{
    private GoudGame game;
    private Dictionary<int, SpriteData> sprites;
    private float speed = 0.05f;

    public Movement(GoudGame gameInstance)
    {
        this.game = gameInstance;
        this.sprites = new Dictionary<int, SpriteData>();
    }

    public void AddSprite(int index, float x, float y, float scaleX, float scaleY, float rotation)
    {
        sprites[index] = new SpriteData { X = x, Y = y, ScaleX = scaleX, ScaleY = scaleY, Rotation = rotation };
        Console.WriteLine("Added sprite with index " + index);
    }

    public void Update()
    {
        foreach (var key in sprites.Keys)
        {
            SpriteData data = sprites[key];

            // Check if WASD keys are pressed
            if (game.IsKeyPressed(87)) data.Y += speed; // W
            if (game.IsKeyPressed(83)) data.Y -= speed; // S
            if (game.IsKeyPressed(65)) data.X -= speed; // A
            if (game.IsKeyPressed(68)) data.X += speed; // D

            sprites[key] = data;
            game.UpdateSprite(key, data.X, data.Y, data.ScaleX, data.ScaleY, data.Rotation);
        }
    }

    private struct SpriteData
    {
        public float X;
        public float Y;
        public float ScaleX;
        public float ScaleY;
        public float Rotation;
    }
}