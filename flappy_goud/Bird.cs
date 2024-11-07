// Bird.cs

using System;

using CsBindgen;
public class Bird
{
    private GoudGame game;
    private int spriteIndex;
    private Movement movement;
    public float X { get; private set; }
    public float Y { get; private set; }

    public Bird(GoudGame game)
    {
        this.game = game;
        this.movement = new Movement(GameConstants.Gravity, GameConstants.JumpStrength);
        this.X = GameConstants.ScreenWidth / 4;
        this.Y = GameConstants.ScreenHeight / 2;
    }

    public void Initialize()
    {
        spriteIndex = game.AddSprite("assets/sprites/bluebird-midflap.png", new SpriteData
        {
            x = X,
            y = Y,
        });
    }

    public void Reset()
    {
        X = GameConstants.ScreenWidth / 4;
        Y = GameConstants.ScreenHeight / 2;
        movement = new Movement(GameConstants.Gravity, GameConstants.JumpStrength);
    }

    public void Update()
    {
        if (game.IsKeyPressed(32)) // Space bar
        {
            movement.Jump();
        }
        movement.ApplyGravity();
        Y += movement.Velocity;

        // Update the bird sprite position
        game.UpdateSprite(spriteIndex, new SpriteData { x = X, y = Y });
    }

    public bool CollidesWith(Pipe pipe)
    {
        return pipe.Intersects(X, Y, GameConstants.BirdWidth, GameConstants.BirdHeight);
    }

    public int PassedPipes(List<Pipe> pipes)
    {
        int count = 0;
        foreach (var pipe in pipes)
        {
            if (pipe.IsPassed(X))
            {
                count++;
            }
        }
        return count;
    }
}