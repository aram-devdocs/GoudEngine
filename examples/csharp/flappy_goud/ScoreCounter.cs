// ScoreCounter.cs

using System.Collections.Generic;

public class ScoreCounter
{
    public int Score { get; private set; }

    // Texture IDs for digits 0-9 (64-bit handles)
    private ulong[] digitTextures;

    // Display settings
    private float xOffset;
    private float yOffset;

    // Digit sprite dimensions (actual image: 24 × 36)
    private const float DigitWidth = 24f;
    private const float DigitHeight = 36f;
    private const float DigitSpacing = 30f;

    public ScoreCounter()
    {
        Score = 0;
        digitTextures = new ulong[10];
    }

    public void Initialize(GoudGame game)
    {
        // Load all digit textures once at initialization (64-bit handles)
        for (int i = 0; i < 10; i++)
        {
            digitTextures[i] = game.LoadTexture($"assets/sprites/{i}.png");
        }

        xOffset = GameConstants.ScreenWidth / 2 - 30; // Center the score counter horizontally
        yOffset = 50; // Offset from the top of the screen
    }

    public void IncrementScore()
    {
        Score++;
    }

    public void ResetScore()
    {
        Score = 0;
    }

    /// <summary>
    /// Draws the score using immediate-mode rendering.
    /// Call this each frame in the render pass.
    /// Note: DrawSprite uses center-based positioning.
    /// </summary>
    public void Draw(GoudGame game)
    {
        // Convert score to a string to separate each digit
        var scoreString = Score.ToString();

        // Draw each digit (center-based positioning)
        for (int i = 0; i < scoreString.Length; i++)
        {
            int digit = scoreString[i] - '0';
            game.DrawSprite(
                digitTextures[digit],
                xOffset + i * DigitSpacing + DigitWidth / 2,
                yOffset + DigitHeight / 2,
                DigitWidth,
                DigitHeight
            );
        }
    }
}
