// Bird.cs

using System;
using CsBindgen;

public class Bird
{
    private readonly GoudGame game;
    private readonly Movement movement;

    private uint spriteIndex;
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
        spriteIndex = game.AddSprite("assets/sprites/bluebird-midflap.png", new SpriteDto
        {
            x = X,
            y = Y,
        });
    }

    public void Reset()
    {
        X = GameConstants.ScreenWidth / 4;
        Y = GameConstants.ScreenHeight / 2;
        movement.Velocity = 0;
    }

    public void Update(float deltaTime)
    {
        if (game.IsKeyPressed(32)) // Space bar for jump
        {
            movement.Jump(deltaTime);
        }

        movement.ApplyGravity(deltaTime);
        float yPosition = Y;
        movement.UpdatePosition(ref yPosition, deltaTime);
        Y = yPosition;

        // Update the bird's position in the game
        game.UpdateSprite(spriteIndex, new SpriteDto { x = X, y = Y, rotation = movement.Rotation });
    }

    public bool CollidesWith(Pipe pipe)
    {
        return pipe.Intersects(X, Y, GameConstants.BirdWidth, GameConstants.BirdHeight);
    }


}