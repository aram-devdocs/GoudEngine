using System;
using CsBindgen;
using System.Collections.Generic;
public class Movement
{
    private GoudGame game;
    private Dictionary<int, SpriteData> sprites;
    private float speed = 7f;
    private float rotation = 0.1f;

    private float scale = 2f;


    public Movement(GoudGame gameInstance)
    {
        this.game = gameInstance;
        this.sprites = new Dictionary<int, SpriteData>();
    }

    public void AddSprite(int index, SpriteData data)

    {

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
            if (game.IsKeyPressed(87)) data.y -= speed; // W
            if (game.IsKeyPressed(83)) data.y += speed; // S
            if (game.IsKeyPressed(65)) data.x -= speed; // A
            if (game.IsKeyPressed(68)) data.x += speed; // D


            // TODO: Rotation and scaling are not working
            // if (game.IsKeyPressed(81)) data.rotation += rotation; // Q
            // if (game.IsKeyPressed(69)) data.rotation -= rotation; // E

            // // scale z x
            // if (game.IsKeyPressed(90))
            // {
            //     data.scale_x += scale;
            //     data.scale_y += scale;
            // }

            // if (game.IsKeyPressed(88))
            // {
            //     data.scale_x -= scale;
            //     data.scale_y -= scale;
            // }

            sprites[key] = data;
            // TODO: https://github.com/aram-devdocs/GoudEngine/issues/7
            game.UpdateSprite(key, data);
        }
    }


}