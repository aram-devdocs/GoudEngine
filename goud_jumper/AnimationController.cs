using System.Collections.Generic;
using CsBindgen;

public class AnimationController
{
    private Dictionary<string, (uint textureId, int frameCount, int frameWidth, int frameHeight)> stateToTextureMap;
    private int currentFrame;
    private float frameTime;
    private float timeSinceLastFrame;
    private string currentState;
    private GoudGame game;

    public AnimationController(
        GoudGame game,
        Dictionary<string, (string texturePath, int frameCount, int frameWidth, int frameHeight)> stateToTextureMap
    )
    {
        this.game = game;
        this.stateToTextureMap = new Dictionary<string, (uint textureId, int frameCount, int frameWidth, int frameHeight)>();

        foreach (var state in stateToTextureMap)
        {
            var textureId = game.CreateTexture(state.Value.texturePath);
            this.stateToTextureMap[state.Key] = (textureId, state.Value.frameCount, state.Value.frameWidth, state.Value.frameHeight);
        }

        this.currentFrame = 0;
        this.frameTime = 0.1f; // Time per frame in seconds
        this.timeSinceLastFrame = 0f;
        this.currentState = string.Empty; // Initialize to an empty string
    }

    public (Rectangle frame, uint textureId) GetFrame(string state, float deltaTime)
    {
        if (!stateToTextureMap.ContainsKey(state))
        {
            throw new ArgumentException($"Invalid state: {state}");
        }

        if (state != currentState)
        {
            ResetAnimationState(state);
        }

        var (textureId, frameCount, frameWidth, frameHeight) = stateToTextureMap[state];
        if (frameCount > 1)
        {
            timeSinceLastFrame += deltaTime;
            if (timeSinceLastFrame >= frameTime)
            {
                currentFrame = (currentFrame + 1) % frameCount;
                timeSinceLastFrame = 0f;
            }
        }

        var frame = new Rectangle
        {
            x = currentFrame * frameWidth,
            y = 0,
            width = frameWidth,
            height = frameHeight
        };

        return (frame, textureId);
    }

    public uint GetInitialTextureId(string state)
    {
        if (!stateToTextureMap.ContainsKey(state))
        {
            throw new ArgumentException($"Invalid state: {state}");
        }

        return stateToTextureMap[state].textureId;
    }

    private void ResetAnimationState(string state)
    {
        currentState = state;
        currentFrame = 0;
        timeSinceLastFrame = 0f;
    }
}
