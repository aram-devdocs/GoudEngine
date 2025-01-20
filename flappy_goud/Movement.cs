public class Movement
{
    public float Velocity { get; set; }
    public float Rotation { get; private set; }

    private readonly float gravity;
    private readonly float jumpStrength;
    private const float RotationSmoothing = 0.03f; // Adjusts how fast the bird's rotation responds

    private float jumpCooldownTimer = 0f;

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
        jumpCooldownTimer -= Math.Max(0, deltaTime);
    }

    public void TryJump(float deltaTime)
    {
        if (jumpCooldownTimer <= 0)
        {
            Jump();
            jumpCooldownTimer = GameConstants.JumpCooldown; // Reset the cooldown timer after jumping
        }
    }

    private void Jump()
    {
        Velocity = 0; // Reset velocity to 0 before applying gravity

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
