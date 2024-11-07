// Movement.cs

public class Movement
{
    public float Velocity { get; set; }
    private readonly float gravity;
    private readonly float jumpStrength;

    public Movement(float gravity, float jumpStrength)
    {
        this.gravity = gravity;
        this.jumpStrength = jumpStrength;
        this.Velocity = 0;
    }

    public void ApplyGravity(float deltaTime)
    {
        Velocity += gravity * deltaTime * GameConstants.TargetFPS;
    }

    public void Jump(float deltaTime)
    {
        Velocity = jumpStrength * GameConstants.TargetFPS;
    }

    public void UpdatePosition(ref float positionY, float deltaTime)
    {
        positionY += Velocity * deltaTime;
    }
}