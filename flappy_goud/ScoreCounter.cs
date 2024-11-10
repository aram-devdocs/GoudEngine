using CsBindgen;

public class ScoreCounter
{
    public int Score { get; private set; }

    public ScoreCounter()
    {
        Score = 0;
    }

    public void IncrementScore()
    {
        Score++;
    }

    public void ResetScore()
    {
        Score = 0;
    }


    public void Update(GoudGame game)
    {
        DrawScore(game);
    }

    public void DrawScore(GoudGame game)
    {
        string scoreString = Score.ToString();
        float x = 10; // Starting x position for drawing the score
        float y = 10; // Starting y position for drawing the score

        foreach (char digit in scoreString)
        {
            string spritePath = $"assets/sprites/{digit}.png";
            game.AddSprite(spritePath, new SpriteDto { x = x, y = y });
            x += 20; // Move to the next position for the next digit
        }


    }


}