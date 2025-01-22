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
        movement = new Movement(GameConstants.Gravity, GameConstants.JumpStrength);
        X = GameConstants.ScreenWidth / 4;
        Y = GameConstants.ScreenHeight / 2;
        animator = new BirdAnimator(game, X, Y);
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
        if (game.IsKeyPressed(32) || game.IsMouseButtonPressed(0))
        {
            movement.TryJump(deltaTime);
        }

        movement.ApplyGravity(deltaTime);
        float yPosition = Y;
        movement.UpdatePosition(ref yPosition, deltaTime);
        Y = yPosition;

        // Update the animator with the new position and rotation
        animator.Update(deltaTime, X, Y, movement.Rotation);
    }

    public uint GetSpriteId()
    {
        return animator.GetSpriteId();
    }
}
