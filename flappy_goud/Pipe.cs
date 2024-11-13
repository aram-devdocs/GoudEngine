// Pipe.cs

using System;
using CsBindgen;

public class Pipe
{
    private readonly GoudGame game;
    public uint topSpriteId;
    public uint bottomSpriteId;
    public float X { get; private set; }
    public float GapY { get; private set; }

    public uint TextureId { get; private set; }

    // Img Dimensions: 52 × 320
    public Pipe(GoudGame game)
    {
        this.game = game;
        X = GameConstants.ScreenWidth;

        // Set the gap to a random vertical position on the screen
        GapY = new Random().Next(GameConstants.PipeGap, (int)GameConstants.ScreenHeight - GameConstants.PipeGap);
        TextureId = game.CreateTexture("assets/sprites/pipe-green.png");

        // Position the top pipe above the screen by its height
        topSpriteId = game.AddSprite(new SpriteCreateDto
        {
            x = X,
            y = GapY - GameConstants.PipeGap - 320, // Adjusted to position the top pipe correctly
            rotation = 180,
            texture_id = TextureId
        });


        bottomSpriteId = game.AddSprite(new SpriteCreateDto
        {
            x = X,
            y = GapY + GameConstants.PipeGap, // Adjusted to position the bottom pipe correctly
            texture_id = TextureId
        });

    }

    public void Update(float deltaTime)
    {
        X -= GameConstants.PipeSpeed * deltaTime * GameConstants.TargetFPS;

        // Update top and bottom pipe positions
        game.UpdateSprite(new SpriteUpdateDto { id = topSpriteId, x = X, y = GapY - GameConstants.PipeGap - 320, rotation = 180, texture_id = TextureId });
        game.UpdateSprite(new SpriteUpdateDto { id = bottomSpriteId, x = X, y = GapY + GameConstants.PipeGap, texture_id = TextureId });
    }

    public bool IsOffScreen()
    {
        return X + GameConstants.PipeWidth < 0;
    }



    public bool IsPassed(float birdX)
    {
        return birdX > X + GameConstants.PipeWidth;
    }

    public bool Remove()
    {
        game.RemoveSprite(topSpriteId);
        game.RemoveSprite(bottomSpriteId);
        return true;
    }
}
