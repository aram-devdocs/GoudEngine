using CsBindgen;
public class PipePair
{
        private GoudGame game;
    private int topPipeIndex;
    private int bottomPipeIndex;
    private float xPosition;
    private float gapY;

    private SpriteData topData;
    private SpriteData bottomData;
    public PipePair(GoudGame game, float xPosition)
    {
        this.game = game;
        this.xPosition = xPosition;
        // TODO: align the gap with the center of the screen
        gapY = new Random().Next(-10, 10); // Randomize the gap position
        this.topData = new SpriteData { x = xPosition, y = gapY, scale_x = 1, scale_y = 1, rotation = 0 };
        this.bottomData = new SpriteData { x = xPosition, y = gapY, scale_x = 1, scale_y = 1, rotation = 0 };


        topPipeIndex = game.AddSprite("assets/sprites/pipe-green.png", topData);
        bottomPipeIndex = game.AddSprite("assets/sprites/pipe-green.png", bottomData);

        Console.WriteLine("Added pipe pair with top index " + topPipeIndex + " and bottom index " + bottomPipeIndex);
    }

    public void Update()
    {
        xPosition -= GameConstants.PipeSpeed;

        // Update the pipe's position
        topData.x = xPosition;
        bottomData.x = xPosition;

        game.UpdateSprite(topPipeIndex, topData);
        game.UpdateSprite(bottomPipeIndex, bottomData);

    }

    public bool IsOffScreen()
    {
        return xPosition < -50;
    }

    public bool CheckCollision(float birdY)
    {
        return birdY < gapY - GameConstants.PipeGap || birdY > gapY + GameConstants.PipeGap;
    }

    public float GetXPosition()
    {
        return xPosition;
    }
}