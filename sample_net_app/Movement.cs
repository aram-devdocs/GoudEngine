using System;
using System.Collections.Generic;
public class Movement
{
    private GoudGame game;
    private Dictionary<int, GoudGame.SpriteData> sprites;
    private float speed = 0.05f;

    public Movement(GoudGame gameInstance)
    {
        this.game = gameInstance;
        this.sprites = new Dictionary<int, GoudGame.SpriteData>();
    }

    public void AddSprite(int index, GoudGame.SpriteData data)

    {

        // sprites[index] = new GoudGame.SpriteData { X = x, Y = y, ScaleX = scaleX, ScaleY = scaleY, Rotation = rotation };
        sprites[index] = data;
        Console.WriteLine("Added sprite with index " + index);
    }

    public void Update()
    {
        foreach (var key in sprites.Keys)
        {
            GoudGame.SpriteData data = sprites[key];

            // Check if WASD keys are pressed
            // TODO: Get current key pressed rather than checking all keys, as this is inefficient and expensive for interop calls
            if (game.IsKeyPressed(87)) data.Y += speed; // W
            if (game.IsKeyPressed(83)) data.Y -= speed; // S
            if (game.IsKeyPressed(65)) data.X -= speed; // A
            if (game.IsKeyPressed(68)) data.X += speed; // D

            sprites[key] = data;
            // TODO: Batch updates and send them to the game engine rather than updating each sprite individually, as interop calls can be expensive to the Rust engine
            game.UpdateSprite(key, data);
        }
    }


}