// Bird.cs

public class Bird
{
    private readonly GoudGame game;
    private readonly Movement movement;
    private readonly BirdAnimator animator;

    public float X { get; private set; }
    public float Y { get; private set; }

    public Bird(GoudGame game)
    {
        this.game = game;
        this.movement = new Movement(GameConstants.Gravity, GameConstants.JumpStrength);
        this.X = GameConstants.ScreenWidth / 4;
        this.Y = GameConstants.ScreenHeight / 2;
        this.animator = new BirdAnimator(game, X, Y);
    }

    public void Initialize()
    {
        animator.Initialize();
    }

    public void Reset()
    {
        X = GameConstants.ScreenWidth / 4;
        Y = GameConstants.ScreenHeight / 2;
        movement.Velocity = 0;
        animator.Reset();
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

        // Update the animator with the new position and rotation
        animator.Update(deltaTime, X, Y, movement.Rotation);
    }

    public bool CollidesWith(Pipe pipe)
    {
        return pipe.Intersects(X, Y, GameConstants.BirdWidth, GameConstants.BirdHeight);
    }
}