using CsBindgen;
using System.Collections.Generic;

public class ScoreCounter
{
    public int Score { get; private set; }

    private Dictionary<string, uint> spritePaths;

    private List<uint> digitIds;

    private uint xOffset { get; set; }
    private uint yOffset { get; set; }

    public ScoreCounter()
    {
        Score = 0;
        spritePaths = new Dictionary<string, uint>();
        digitIds = new List<uint>();

    }




    public void Initialize(GoudGame game)
    {
        spritePaths = new Dictionary<string, uint>
        {
            { "assets/sprites/0.png", game.CreateTexture("assets/sprites/0.png") },
            { "assets/sprites/1.png", game.CreateTexture("assets/sprites/1.png") },
            { "assets/sprites/2.png", game.CreateTexture("assets/sprites/2.png") },
            { "assets/sprites/3.png", game.CreateTexture("assets/sprites/3.png") },
            { "assets/sprites/4.png", game.CreateTexture("assets/sprites/4.png") },
            { "assets/sprites/5.png", game.CreateTexture("assets/sprites/5.png") },
            { "assets/sprites/6.png", game.CreateTexture("assets/sprites/6.png") },
            { "assets/sprites/7.png", game.CreateTexture("assets/sprites/7.png") },
            { "assets/sprites/8.png", game.CreateTexture("assets/sprites/8.png") },
            { "assets/sprites/9.png", game.CreateTexture("assets/sprites/9.png") }
        };

        xOffset = GameConstants.ScreenWidth / 2 - 30; // Center the score counter horizontally
        yOffset = 50; // Offset from the top of the screen

        // score will start with single digit, so we just need to create one digit in the sprite list
        digitIds = new List<uint> { game.AddSprite(new SpriteCreateDto { z_layer = 1, x = xOffset, y = yOffset, texture_id = spritePaths["assets/sprites/0.png"] }), };
    }
    public void IncrementScore(GoudGame game)
    {
        Score++;
    }

    public void ResetScore(GoudGame game)
    {
        Score = 0;
        // set all sprites to 0
        for (int i = 0; i < digitIds.Count; i++)
        {
            if (i == 0)
            {
                game.UpdateSprite(new SpriteUpdateDto { id = digitIds[i], x = xOffset, y = yOffset, texture_id = spritePaths["assets/sprites/0.png"] });
            }
            else
            {
                game.RemoveSprite(digitIds[i]);
                // remove the sprite from the list
                digitIds.RemoveAt(i);
            }
        }
    }


    public void Update(GoudGame game)
    {
        // Convert score to a string to separate each digit
        var scoreString = Score.ToString();
        int length = scoreString.Length;

        // Ensure there are enough sprites to represent each digit in the score
        while (digitIds.Count < length)
        {
            uint newDigitId = game.AddSprite(new SpriteCreateDto
            {
                z_layer = 1,
                x = xOffset + digitIds.Count * 30, // Offset each digit horizontally
                y = yOffset,
                texture_id = spritePaths["assets/sprites/0.png"] // Initial texture
            });
            digitIds.Add(newDigitId);
        }

        // Update each digit sprite's texture and position based on current score
        for (int i = 0; i < length; i++)
        {
            char digitChar = scoreString[i];
            string digitPath = $"assets/sprites/{digitChar}.png";
            uint textureId = spritePaths[digitPath];

            // Update the sprite's texture to the corresponding digit
            game.UpdateSprite(new SpriteUpdateDto
            {
                texture_id = textureId,
                id = digitIds[i],
                x = xOffset + i * 30, // Adjust position for each digit
                y = yOffset
            });
        }

    }


}
