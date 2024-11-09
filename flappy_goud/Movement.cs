// Movement.cs

public class Movement
{
    public float Velocity { get; set; }

    public float Rotation { get; private set; }
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
        // update the rotation of the bird going up or down
        Rotation = Math.Min(Math.Max(-45, Velocity * 3), 45);
    }
}