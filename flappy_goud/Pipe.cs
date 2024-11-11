// Pipe.cs

using System;
using CsBindgen;

public class Pipe
{
    private readonly GoudGame game;
    private uint topSpriteId;
    private uint bottomSpriteId;
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

        Console.WriteLine($"Debug top_sprite_id: {topSpriteId}");

        bottomSpriteId = game.AddSprite(new SpriteCreateDto
        {
            x = X,
            y = GapY + GameConstants.PipeGap, // Adjusted to position the bottom pipe correctly
            texture_id = TextureId
        });

        Console.WriteLine($"Debug bottom_sprite_id: {bottomSpriteId}");
    }

    public void Update(float deltaTime)
    {
        X -= GameConstants.PipeSpeed * deltaTime * GameConstants.TargetFPS;

        // Update top and bottom pipe positions
        game.UpdateSprite(topSpriteId, new SpriteUpdateDto { x = X, y = GapY - GameConstants.PipeGap - 320, rotation = 180, texture_id = TextureId, debug = true });
        game.UpdateSprite(bottomSpriteId, new SpriteUpdateDto { x = X, y = GapY + GameConstants.PipeGap, texture_id = TextureId, debug = true });
    }

    public bool IsOffScreen()
    {
        return X + GameConstants.PipeWidth < 0;
    }

    public bool Intersects(float birdX, float birdY, int birdWidth, int birdHeight)
    {
        return (birdX < X + GameConstants.PipeWidth && birdX + birdWidth > X &&
                (birdY < GapY || birdY + birdHeight > GapY + GameConstants.PipeGap));
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
