// BirdAnimator.cs

using System;
using System.Collections.Generic;

using CsBindgen;
public class BirdAnimator
{
    private readonly GoudGame game;
    private readonly List<string> spritePaths;
    private int currentFrame;
    private float animationTime;
    private readonly float frameDuration;
    private float initialX;
    private float initialY;

    public uint SpriteIndex { get; private set; }

    public BirdAnimator(GoudGame game, float x, float y, float frameDuration = 0.1f)
    {
        this.game = game;
        this.frameDuration = frameDuration;
        animationTime = 0;
        currentFrame = 0;
        initialX = x;
        initialY = y;

        // Load the animation frames
        spritePaths = new List<string>
        {
            "assets/sprites/bluebird-downflap.png",
            "assets/sprites/bluebird-midflap.png",
            "assets/sprites/bluebird-upflap.png"
        };
    }

    public void Initialize()
    {
        // Set up the initial sprite
        SpriteIndex = game.AddSprite(spritePaths[currentFrame], new SpriteDto { x = initialX, y = initialY });
    }

    public void Update(float deltaTime, float x, float y, float rotation)
    {
        animationTime += deltaTime;

        // Update frame if the animation time exceeds the frame duration
        if (animationTime >= frameDuration)
        {
            currentFrame = (currentFrame + 1) % spritePaths.Count;
            game.RemoveSprite(SpriteIndex);
            SpriteIndex = game.AddSprite(spritePaths[currentFrame], new SpriteDto { x = x, y = y });
            animationTime = 0;
        }
        else
        {
            // Update only position and rotation if within the same frame
            game.UpdateSprite(SpriteIndex, new SpriteDto { x = x, y = y, rotation = rotation });
        }
    }

    public void Reset()
    {
        currentFrame = 0;
        animationTime = 0;
        game.UpdateSprite(SpriteIndex, new SpriteDto { x = initialX, y = initialY });
    }
}