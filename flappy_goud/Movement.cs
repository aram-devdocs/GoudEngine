// Movement.cs

public class Movement
{
    public float Velocity { get; private set; }
    private float gravity;
    private float jumpStrength;

    public Movement(float gravity, float jumpStrength)
    {
        this.gravity = gravity;
        this.jumpStrength = jumpStrength;
        this.Velocity = 0;
    }

    public void ApplyGravity()
    {
        Velocity += gravity * GameConstants.DeltaTime;
    }

    public void Jump()
    {
        Velocity = jumpStrength;
    }
}