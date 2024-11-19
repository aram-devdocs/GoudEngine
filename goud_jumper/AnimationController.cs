using System;
using System.Collections.Generic;
using CsBindgen;

/// <summary>
/// A utility class for controlling sprite sheet animations across different states,
/// with individual speed scaling and looping control per state.
/// </summary>
public class AnimationController
{
    private class AnimationState
    {
        public uint TextureId { get; }
        public int FrameCount { get; }
        public int FrameWidth { get; }
        public int FrameHeight { get; }
        public float FrameTime { get; }
        public float SpeedScale { get; }
        public bool ShouldLoop { get; }

        public AnimationState(
            uint textureId,
            int frameCount,
            int frameWidth,
            int frameHeight,
            float frameTime,
            float speedScale,
            bool shouldLoop
        )
        {
            TextureId = textureId;
            FrameCount = frameCount;
            FrameWidth = frameWidth;
            FrameHeight = frameHeight;
            FrameTime = frameTime;
            SpeedScale = speedScale;
            ShouldLoop = shouldLoop;
        }
    }

    private readonly Dictionary<string, AnimationState> stateToAnimationMap;
    private int currentFrame;
    private float timeSinceLastFrame;
    private string currentState;
    private readonly GoudGame game;

    /// <summary>
    /// Initializes a new instance of the <see cref="AnimationController"/> class.
    /// </summary>
    /// <param name="game">The game instance.</param>
    /// <param name="stateConfigurations">A dictionary mapping state names to their texture paths and animation parameters. The parameters are:
    /// <list type="bullet">
    /// <item><description>Texture path</description></item>
    /// <item><description>Frame count</description></item>
    /// <item><description>Frame width</description></item>
    /// <item><description>Frame height</description></item>
    /// <item><description>Frame time</description></item>
    /// <item><description>Speed scale</description></item>
    /// <item><description>Should loop</description></item>
    /// </list>
    /// </param>
    ///
    /// <exception cref="ArgumentNullException">Thrown when <paramref name="game"/> or <paramref name="stateConfigurations"/> is null.</exception>
    public AnimationController(
        GoudGame game,
        Dictionary<
            string,
            (
                string texturePath,
                int frameCount,
                int frameWidth,
                int frameHeight,
                float frameTime,
                float speedScale,
                bool shouldLoop
            )
        > stateConfigurations
    )
    {
        this.game = game ?? throw new ArgumentNullException(nameof(game));
        if (stateConfigurations == null)
            throw new ArgumentNullException(nameof(stateConfigurations));

        stateToAnimationMap = new Dictionary<string, AnimationState>();

        foreach (var state in stateConfigurations)
        {
            if (string.IsNullOrEmpty(state.Key))
            {
                throw new ArgumentException(
                    "State name cannot be null or empty.",
                    nameof(stateConfigurations)
                );
            }

            var (
                texturePath,
                frameCount,
                frameWidth,
                frameHeight,
                frameTime,
                speedScale,
                shouldLoop
            ) = state.Value;

            if (string.IsNullOrEmpty(texturePath))
            {
                throw new ArgumentException(
                    $"Texture path cannot be null or empty for state '{state.Key}'."
                );
            }

            if (frameCount <= 0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(frameCount),
                    "Frame count must be greater than zero."
                );
            }

            if (frameWidth <= 0 || frameHeight <= 0)
            {
                throw new ArgumentOutOfRangeException(
                    "Frame dimensions must be greater than zero."
                );
            }

            if (frameTime <= 0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(frameTime),
                    "Frame time must be greater than zero."
                );
            }

            if (speedScale <= 0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(speedScale),
                    "Speed scale must be greater than zero."
                );
            }

            var textureId = game.CreateTexture(texturePath);
            var animationState = new AnimationState(
                textureId,
                frameCount,
                frameWidth,
                frameHeight,
                frameTime,
                speedScale,
                shouldLoop
            );
            stateToAnimationMap[state.Key] = animationState;
        }

        currentFrame = 0;
        timeSinceLastFrame = 0f;
        currentState = string.Empty;
    }

    /// <summary>
    /// Gets the current frame rectangle and texture ID for the specified state.
    /// </summary>
    /// <param name="state">The animation state.</param>
    /// <param name="deltaTime">The time elapsed since the last update.</param>
    /// <returns>A tuple containing the frame rectangle and texture ID.</returns>
    /// <exception cref="ArgumentException">Thrown when the state is invalid.</exception>
    public (Rectangle frame, uint textureId) GetFrame(string state, float deltaTime)
    {
        if (!stateToAnimationMap.TryGetValue(state, out var animationState))
        {
            throw new ArgumentException($"Invalid state: {state}", nameof(state));
        }

        if (state != currentState)
        {
            ResetAnimationState(state);
            animationState = stateToAnimationMap[state];
        }

        if (animationState.FrameCount > 1)
        {
            timeSinceLastFrame += deltaTime * animationState.SpeedScale;
            if (timeSinceLastFrame >= animationState.FrameTime)
            {
                timeSinceLastFrame = 0f;

                if (currentFrame < animationState.FrameCount - 1)
                {
                    currentFrame++;
                }
                else if (animationState.ShouldLoop)
                {
                    currentFrame = 0;
                }
                else
                {
                    currentFrame = animationState.FrameCount - 1; // Stay on the last frame
                }
            }
        }

        var frame = new Rectangle
        {
            x = currentFrame * animationState.FrameWidth,
            y = 0,
            width = animationState.FrameWidth,
            height = animationState.FrameHeight
        };

        return (frame, animationState.TextureId);
    }

    /// <summary>
    /// Gets the initial texture ID for the specified state.
    /// </summary>
    /// <param name="state">The animation state.</param>
    /// <returns>The texture ID associated with the state.</returns>
    /// <exception cref="ArgumentException">Thrown when the state is invalid.</exception>
    public uint GetInitialTextureId(string state)
    {
        if (!stateToAnimationMap.TryGetValue(state, out var animationState))
        {
            throw new ArgumentException($"Invalid state: {state}", nameof(state));
        }

        return animationState.TextureId;
    }

    /// <summary>
    /// Resets the animation to its initial state.
    /// </summary>
    /// <param name="state">The animation state to reset to.</param>
    private void ResetAnimationState(string state)
    {
        currentState = state;
        currentFrame = 0;
        timeSinceLastFrame = 0f;
    }
}
