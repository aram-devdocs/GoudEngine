using System.Collections.Generic;
using CsBindgen;

public class AnimationController
{
    private Dictionary<string, List<(int x, int y, int? width, int? height)>> stateToPositionMap;
    private int defaultFrameWidth;
    private int defaultFrameHeight;
    private int currentFrame;
    private float frameTime;
    private float timeSinceLastFrame;
    private string currentState;

    public AnimationController(
        int defaultFrameWidth,
        int defaultFrameHeight,
        Dictionary<string, List<(int x, int y, int? width, int? height)>> stateToPositionMap
    )
    {
        this.defaultFrameWidth = defaultFrameWidth;
        this.defaultFrameHeight = defaultFrameHeight;
        this.stateToPositionMap = stateToPositionMap;
        this.currentFrame = 0;
        this.frameTime = 0.1f; // Time per frame in seconds
        this.timeSinceLastFrame = 0f;
        this.currentState = null;
    }

    public Rectangle GetFrame(string state, float deltaTime)
    {
        if (!stateToPositionMap.ContainsKey(state) || stateToPositionMap[state].Count == 0)
        {
            throw new ArgumentException($"Invalid state: {state}");
        }

        if (state != currentState)
        {
            ResetAnimationState(state);
        }

        var positions = stateToPositionMap[state];
        if (positions.Count > 1)
        {
            timeSinceLastFrame += deltaTime;
            if (timeSinceLastFrame >= frameTime)
            {
                currentFrame = (currentFrame + 1) % positions.Count;
                timeSinceLastFrame = 0f;
            }
        }
        var position = positions[currentFrame];
        return new Rectangle
        {
            x = position.x,
            y = position.y,
            width = position.width ?? defaultFrameWidth,
            height = position.height ?? defaultFrameHeight
        };
    }

    private void ResetAnimationState(string state)
    {
        currentState = state;
        currentFrame = 0;
        timeSinceLastFrame = 0f;
    }
}
