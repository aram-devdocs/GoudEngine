public class Bird
{
    private GoudGame game;
    private int birdSpriteIndex;
    private float velocity = 0;
    private float yPosition = 0f; // Starting position

    private GoudGame.SpriteData data;

    public Bird(GoudGame game, float xPosition)
    {
        this.game = game;
        this.data = new GoudGame.SpriteData { X = xPosition, Y = yPosition, ScaleX = 0.3f, ScaleY = 0.3f, Rotation = 0 };
        this.birdSpriteIndex = game.AddSprite("assets/sprites/bluebird-midflap.png", data);

    }

    public void Update()
    {
        // Apply gravity
        velocity += GameConstants.Gravity;
        yPosition += velocity;

        // Jump if spacebar is pressed
        if (game.IsKeyPressed(32)) // Spacebar
        {
            velocity = GameConstants.JumpStrength;
        }

        // update data
        data.Y = yPosition;
        data.Rotation = velocity * 5; // Rotate the bird based on velocity



        // Update the bird's position
        game.UpdateSprite(birdSpriteIndex, data);
    }

    public bool HasHitGround()
    {
        return yPosition >= GameConstants.GroundYPosition;
    }

    public float GetYPosition()
    {
        return yPosition;
    }
}