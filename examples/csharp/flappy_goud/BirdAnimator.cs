// BirdAnimator.cs

using System;
using System.Collections.Generic;

public class BirdAnimator
{
    private readonly GoudGame game;
    private List<ulong> frameTextures;

    private int currentFrame;
    private float animationTime;
    private readonly float frameDuration;
    private float initialX;
    private float initialY;

    // Current state for drawing
    private float currentX;
    private float currentY;
    private float currentRotation;

    // Bird sprite dimensions
    private const float BirdWidth = 34f;
    private const float BirdHeight = 24f;

    public BirdAnimator(GoudGame game, float x, float y, float frameDuration = 0.1f)
    {
        this.game = game;
        this.frameDuration = frameDuration;
        animationTime = 0;
        currentFrame = 0;
        initialX = x;
        initialY = y;
        currentX = x;
        currentY = y;
        currentRotation = 0;
        frameTextures = new List<ulong>();
    }

    public void Initialize()
    {
        // Load textures for animation frames (64-bit handles)
        frameTextures = new List<ulong>
        {
            game.LoadTexture("assets/sprites/bluebird-downflap.png"),
            game.LoadTexture("assets/sprites/bluebird-midflap.png"),
            game.LoadTexture("assets/sprites/bluebird-upflap.png")
        };
    }

    public void Update(float deltaTime, float x, float y, float rotation)
    {
        animationTime += deltaTime;
        if (animationTime >= frameDuration)
        {
            currentFrame = (currentFrame + 1) % frameTextures.Count;
            animationTime = 0;
        }

        // Store current state for drawing
        currentX = x;
        currentY = y;
        currentRotation = rotation;
    }

    /// <summary>
    /// Draws the bird sprite using immediate-mode rendering.
    /// Call this each frame in the render pass.
    /// Note: DrawSprite uses center-based positioning.
    /// </summary>
    public void Draw()
    {
        if (frameTextures.Count == 0) return;
        
        // Bird position is already the center of the bird
        game.DrawSprite(
            frameTextures[currentFrame],
            currentX + BirdWidth / 2,
            currentY + BirdHeight / 2,
            BirdWidth,
            BirdHeight,
            currentRotation * (float)System.Math.PI / 180f  // Convert degrees to radians
        );
    }

    public void Reset()
    {
        currentFrame = 0;
        animationTime = 0;
        currentX = initialX;
        currentY = initialY;
        currentRotation = 0;
    }

    /// <summary>
    /// Gets the current texture ID for collision detection purposes.
    /// </summary>
    public ulong GetCurrentTextureId()
    {
        return frameTextures.Count > 0 ? frameTextures[currentFrame] : 0;
    }
}
