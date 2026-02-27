// Pipe.cs

using System;

public class Pipe
{
    private readonly GoudGame game;
    public float X { get; private set; }
    public float GapY { get; private set; }

    // Pipe sprite dimensions (actual image: 52 × 320)
    public const float PipeWidth = 52f;
    public const float PipeHeight = 320f;

    // Calculated positions for collision detection
    public float TopPipeY => GapY - GameConstants.PipeGap - PipeHeight;
    public float BottomPipeY => GapY + GameConstants.PipeGap;

    public Pipe(GoudGame game)
    {
        this.game = game;
        X = GameConstants.ScreenWidth;

        // Set the gap to a random vertical position on the screen
        GapY = new Random().Next(
            GameConstants.PipeGap,
            (int)GameConstants.ScreenHeight - GameConstants.PipeGap
        );
    }

    public void Update(float deltaTime)
    {
        X -= GameConstants.PipeSpeed * deltaTime * GameConstants.TargetFPS;
    }

    /// <summary>
    /// Draws the pipe (both top and bottom) using immediate-mode rendering.
    /// </summary>
    /// <param name="pipeTextureId">Shared texture ID for pipes (64-bit handle)</param>
    public void Draw(ulong pipeTextureId)
    {
        // Draw top pipe (rotated 180 degrees)
        // Center-based positioning: offset by half width/height
        game.DrawSprite(
            pipeTextureId,
            X + PipeWidth / 2,
            TopPipeY + PipeHeight / 2,
            PipeWidth,
            PipeHeight,
            (float)Math.PI  // 180 degrees in radians
        );

        // Draw bottom pipe (no rotation)
        game.DrawSprite(
            pipeTextureId,
            X + PipeWidth / 2,
            BottomPipeY + PipeHeight / 2,
            PipeWidth,
            PipeHeight,
            0f
        );
    }

    public bool IsOffScreen()
    {
        return X + GameConstants.PipeWidth < 0;
    }

    public bool IsPassed(float birdX)
    {
        return birdX > X + GameConstants.PipeWidth;
    }

    /// <summary>
    /// Checks if the bird collides with this pipe.
    /// Uses simple AABB collision detection.
    /// </summary>
    public bool CollidesWithBird(float birdX, float birdY, float birdWidth, float birdHeight)
    {
        // Check collision with top pipe
        if (CheckAABBCollision(
            birdX, birdY, birdWidth, birdHeight,
            X, TopPipeY, PipeWidth, PipeHeight))
        {
            return true;
        }

        // Check collision with bottom pipe
        if (CheckAABBCollision(
            birdX, birdY, birdWidth, birdHeight,
            X, BottomPipeY, PipeWidth, PipeHeight))
        {
            return true;
        }

        return false;
    }

    private static bool CheckAABBCollision(
        float x1, float y1, float w1, float h1,
        float x2, float y2, float w2, float h2)
    {
        return x1 < x2 + w2 &&
               x1 + w1 > x2 &&
               y1 < y2 + h2 &&
               y1 + h1 > y2;
    }
}
