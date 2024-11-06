using System;
using CsBindgen;
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

    public void AddSprite(int index, SpriteData data)

    {

        // sprites[index] = new SpriteData { X = x, Y = y, ScaleX = scaleX, ScaleY = scaleY, Rotation = rotation };
        sprites[index] = data;
        Console.WriteLine("Added sprite with index " + index);
    }

    public void Update()
    {
        foreach (var key in sprites.Keys)
        {
            SpriteData data = sprites[key];

            // Check if WASD keys are pressed
            // TODO: https://github.com/aram-devdocs/GoudEngine/issues/6
            if (game.IsKeyPressed(87)) data.y += speed; // W
            if (game.IsKeyPressed(83)) data.y -= speed; // S
            if (game.IsKeyPressed(65)) data.x -= speed; // A
            if (game.IsKeyPressed(68)) data.x += speed; // D

            if (game.IsKeyPressed(81)) data.rotation += speed; // Q
            if (game.IsKeyPressed(69)) data.rotation -= speed; // E

            // scale z x
            if (game.IsKeyPressed(90))
            {
                data.scale_x += speed;
                data.scale_y += speed;
            }

            if (game.IsKeyPressed(88))
            {
                data.scale_x -= speed;
                data.scale_y -= speed;
            }

            sprites[key] = data;
            // TODO: https://github.com/aram-devdocs/GoudEngine/issues/7
            game.UpdateSprite(key, data);
        }
    }


}