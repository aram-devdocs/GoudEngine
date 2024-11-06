using CsBindgen;
public class Bird
{
    private GoudGame game;
    public int birdSpriteIndex;
    private float velocity = 0;

    // TODO: https://github.com/aram-devdocs/GoudEngine/issues/3
    private float yPosition = 0f; // Starting position

    public SpriteData data;

    public Bird(GoudGame game, float xPosition)
    {
        this.game = game;
        this.data = new SpriteData { x = xPosition, y = yPosition, scale_x = 0.3f, scale_y = 0.3f, rotation = 0 };
        this.birdSpriteIndex = game.AddSprite("assets/sprites/bluebird-midflap.png", data);

    }

    public void Update()
    {

        // TODO: https://github.com/aram-devdocs/GoudEngine/issues/4
        // Apply gravity
        // velocity += GameConstants.Gravity;

        // // Jump if spacebar is pressed
        // if (game.IsKeyPressed(32)) // Spacebar
        // {
        //     velocity = GameConstants.JumpStrength;
        // }


        // yPosition += velocity;

        // // update data
        // data.Y = yPosition;
        // data.Rotation = velocity * 5; // Rotate the bird based on velocity



        // // Update the bird's position
        // game.UpdateSprite(birdSpriteIndex, data);
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