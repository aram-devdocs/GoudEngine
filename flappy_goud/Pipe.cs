// Pipe.cs

using System;
using CsBindgen;

public class Pipe
{
    private readonly GoudGame game;
    private int topSpriteId;
    private int bottomSpriteId;
    public float X { get; private set; }
    public float GapY { get; private set; }

    public Pipe(GoudGame game)
    {
        this.game = game;
        this.X = GameConstants.ScreenWidth;
        this.GapY = new Random().Next(GameConstants.PipeGap, (int)GameConstants.ScreenHeight - GameConstants.PipeGap);

        topSpriteId = game.AddSprite("assets/sprites/pipe-green.png", new SpriteDto
        {
            x = X,
            y = GapY - GameConstants.PipeGap - GameConstants.PipeWidth,
        });

        bottomSpriteId = game.AddSprite("assets/sprites/pipe-green.png", new SpriteDto
        {
            x = X,
            y = GapY + GameConstants.PipeGap,
        });
    }

    public void Update(float deltaTime)
    {
        X -= GameConstants.PipeSpeed * deltaTime * GameConstants.TargetFPS;

        // Update top and bottom pipe positions
        game.UpdateSprite(topSpriteId, new SpriteDto { x = X });
        game.UpdateSprite(bottomSpriteId, new SpriteDto { x = X });
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