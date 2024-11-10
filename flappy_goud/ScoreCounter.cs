using CsBindgen;
using System.Collections.Generic;

public class ScoreCounter
{
    public int Score { get; private set; }
    private List<int> spriteIndices;

    public ScoreCounter()
    {
        Score = 0;
        spriteIndices = new List<int>();
    }

    public void IncrementScore(GoudGame game)
    {
        RemoveOldSprites(game);
        Score++;
        DrawScore(game);
    }

    public void ResetScore(GoudGame game)
    {
        RemoveOldSprites(game);
        Score = 0;
        DrawScore(game);
    }

    public void Update(GoudGame game)
    {
        DrawScore(game);
    }

    private void DrawScore(GoudGame game)
    {
        string scoreString = Score.ToString();
        float x = 10; // Starting x position for drawing the score
        float y = 10; // Starting y position for drawing the score

        foreach (char digit in scoreString)
        {
            string spritePath = $"assets/sprites/{digit}.png";
            int spriteIndex = (int)game.AddSprite(spritePath, new SpriteDto { x = x, y = y });
            spriteIndices.Add(spriteIndex);
            x += 20; // Move to the next position for the next digit
        }
    }

    private void RemoveOldSprites(GoudGame game)
    {
        foreach (int index in spriteIndices)
        {
            game.RemoveSprite((uint)index);
        }
        spriteIndices.Clear();
    }
}