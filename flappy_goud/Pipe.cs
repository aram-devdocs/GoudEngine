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

    // Img Dimensions: 52 × 320
    public Pipe(GoudGame game)
    {
        this.game = game;
        X = GameConstants.ScreenWidth;

        // Set the gap to a random vertical position on the screen
        GapY = new Random().Next(GameConstants.PipeGap, (int)GameConstants.ScreenHeight - GameConstants.PipeGap);

        // Position the top pipe above the screen by its height
        topSpriteId = game.AddSprite("assets/sprites/pipe-green.png", new SpriteDto
        {
            x = X,
            y = GapY - GameConstants.PipeGap - 320, // Adjusted to position the top pipe correctly
            rotation = 180
        });

        bottomSpriteId = game.AddSprite("assets/sprites/pipe-green.png", new SpriteDto
        {
            x = X,
            y = GapY + GameConstants.PipeGap // Adjusted to position the bottom pipe correctly
        });
    }

    public void Update(float deltaTime)
    {
        X -= GameConstants.PipeSpeed * deltaTime * GameConstants.TargetFPS;

        // Update top and bottom pipe positions
        game.UpdateSprite(topSpriteId, new SpriteDto { x = X, y = GapY - GameConstants.PipeGap - 320, rotation = 180 });
        game.UpdateSprite(bottomSpriteId, new SpriteDto { x = X, y = GapY + GameConstants.PipeGap });
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
