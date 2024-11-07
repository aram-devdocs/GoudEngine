// Pipe.cs
using CsBindgen;
using System;


public class Pipe
{
    private GoudGame game;
    private int topSpriteIndex;
    private int bottomSpriteIndex;
    public float X { get; private set; }
    public float GapY { get; private set; }

    public Pipe(GoudGame game)
    {
        this.game = game;
        this.X = GameConstants.ScreenWidth;
        this.GapY = new Random().Next(GameConstants.PipeGap, (int)GameConstants.ScreenHeight - GameConstants.PipeGap);

        // Top Pipe
        topSpriteIndex = game.AddSprite("assets/sprites/pipe-green.png", new SpriteData
        {
            x = X,
            y = 0,
        });

        // Bottom Pipe
        bottomSpriteIndex = game.AddSprite("assets/sprites/pipe-green.png", new SpriteData
        {
            x = X,
            y = GapY + GameConstants.PipeGap,
        });




    }

    public void Update()
    {

    }

    public bool IsOffScreen()
    {
        return false;
    }

    public bool Intersects(float birdX, float birdY, int birdWidth, int birdHeight)
    {
        return false;
    }

    public bool IsPassed(float birdX)
    {
        return false;

    }
}