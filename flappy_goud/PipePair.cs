public class PipePair
{
    private GoudGame game;
    private int topPipeIndex;
    private int bottomPipeIndex;
    private float xPosition;
    private float gapY;

    private GoudGame.SpriteData topData;
    private GoudGame.SpriteData bottomData;
    public PipePair(GoudGame game, float xPosition)
    {
        this.game = game;
        this.xPosition = xPosition;
        gapY = new Random().Next(150, 350); // Randomize the gap position
        this.topData = new GoudGame.SpriteData { X = xPosition, Y = gapY, ScaleX = 1, ScaleY = 1, Rotation = 0 };
        this.bottomData = new GoudGame.SpriteData { X = xPosition, Y = gapY, ScaleX = 1, ScaleY = 1, Rotation = 0 };


        topPipeIndex = game.AddSprite("assets/sprites/pipe-green.png", topData);
        bottomPipeIndex = game.AddSprite("assets/sprites/pipe-green.png", bottomData);

        Console.WriteLine("Added pipe pair with top index " + topPipeIndex + " and bottom index " + bottomPipeIndex);
    }

    public void Update()
    {
        xPosition -= GameConstants.PipeSpeed;

        // Update the pipe's position
        topData.X = xPosition;
        bottomData.X = xPosition;

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