public class Movement
{
    public float Velocity { get; set; }
    public float Rotation { get; private set; }

    private readonly float gravity;
    private readonly float jumpStrength;
    private const float RotationSmoothing = 0.03f; // Adjusts how fast the bird's rotation responds

    public Movement(float gravity, float jumpStrength)
    {
        this.gravity = gravity;
        this.jumpStrength = jumpStrength;
        Velocity = 0;
        Rotation = 0;
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

        // Gradually update rotation based on velocity, with smoothing
        float targetRotation = Math.Clamp(Velocity * 3, -45, 45); // Controls max rotation angle based on velocity
        Rotation += (targetRotation - Rotation) * RotationSmoothing;
    }
}