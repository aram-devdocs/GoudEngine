using System;
using System.Collections.Generic;
using CsBindgen;

// TODO
// - Right now when manually assining a sprite, X Y is on the bottom left rather than top left. this needs to be normalized as we render sprites from the top left



/// <summary>
/// A utility class for controlling sprite sheet animations across different states,
/// supporting both grid-based and frame list-based spritesheets.
/// </summary>
public class AnimationController
{
    private class AnimationState
    {
        public uint TextureId { get; }
        public float FrameTime { get; }
        public float SpeedScale { get; }
        public bool ShouldLoop { get; }
        public List<Rectangle> Frames { get; }

        public AnimationState(
            uint textureId,
            List<Rectangle> frames,
            float frameTime,
            float speedScale,
            bool shouldLoop
        )
        {
            TextureId = textureId;
            Frames = frames;
            FrameTime = frameTime;
            SpeedScale = speedScale;
            ShouldLoop = shouldLoop;
        }
    }

    private readonly Dictionary<string, AnimationState> stateToAnimationMap;
    private readonly Dictionary<string, uint> textureIdMap;
    private int currentFrameIndex;
    private float timeSinceLastFrame;
    private string currentState;
    private readonly GoudGame game;

    /// <summary>
    /// Initializes a new instance of the <see cref="AnimationController"/> class.
    /// </summary>
    /// <param name="game">The game instance.</param>
    /// <param name="stateConfigurations">
    /// A dictionary mapping state names to their animation configurations.
    /// Each configuration can be grid-based or frame list-based.
    /// </param>
    /// <exception cref="ArgumentNullException">Thrown when <paramref name="game"/> or <paramref name="stateConfigurations"/> is null.</exception>
    public AnimationController(
        GoudGame game,
        Dictionary<string, AnimationStateConfig> stateConfigurations
    )
    {
        this.game = game ?? throw new ArgumentNullException(nameof(game));
        if (stateConfigurations == null)
            throw new ArgumentNullException(nameof(stateConfigurations));

        stateToAnimationMap = new Dictionary<string, AnimationState>();
        textureIdMap = new Dictionary<string, uint>();

        foreach (var kvp in stateConfigurations)
        {
            var stateName = kvp.Key;
            var config = kvp.Value;

            if (string.IsNullOrEmpty(stateName))
            {
                throw new ArgumentException(
                    "State name cannot be null or empty.",
                    nameof(stateConfigurations)
                );
            }

            if (string.IsNullOrEmpty(config.TexturePath))
            {
                throw new ArgumentException(
                    $"Texture path cannot be null or empty for state '{stateName}'."
                );
            }

            if (config.FrameTime <= 0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(config.FrameTime),
                    "Frame time must be greater than zero."
                );
            }

            if (config.SpeedScale <= 0)
            {
                throw new ArgumentOutOfRangeException(
                    nameof(config.SpeedScale),
                    "Speed scale must be greater than zero."
                );
            }

            // Load texture or reuse if already loaded
            if (!textureIdMap.TryGetValue(config.TexturePath, out uint textureId))
            {
                textureId = game.CreateTexture(config.TexturePath);
                textureIdMap[config.TexturePath] = textureId;
            }

            List<Rectangle> frames;

            if (config.Frames != null && config.Frames.Count > 0)
            {
                // Frame List Mode
                frames = config.Frames;
            }
            else if (
                config.FrameWidth.HasValue
                && config.FrameHeight.HasValue
                && config.StartFrameIndex.HasValue
                && config.FrameCount.HasValue
                && config.Columns.HasValue
            )
            {
                // Grid-Based Mode with specified frame indices
                frames = GenerateGridFrames(
                    config.FrameWidth.Value,
                    config.FrameHeight.Value,
                    config.Columns.Value,
                    config.StartFrameIndex.Value,
                    config.FrameCount.Value
                );
            }
            else
            {
                throw new ArgumentException(
                    $"State '{stateName}' must have either Frames or FrameWidth, FrameHeight, StartFrameIndex, FrameCount, and Columns specified."
                );
            }

            var animationState = new AnimationState(
                textureId,
                frames,
                config.FrameTime,
                config.SpeedScale,
                config.ShouldLoop
            );

            stateToAnimationMap[stateName] = animationState;
        }

        currentFrameIndex = 0;
        timeSinceLastFrame = 0f;
        currentState = string.Empty;
    }

    /// <summary>
    /// Generates frames for grid-based spritesheets using start index and frame count.
    /// </summary>
    private List<Rectangle> GenerateGridFrames(
        int frameWidth,
        int frameHeight,
        int columns,
        int startFrameIndex,
        int frameCount
    )
    {
        var frames = new List<Rectangle>();
        for (int i = startFrameIndex; i < startFrameIndex + frameCount; i++)
        {
            int x = (i % columns) * frameWidth;
            int y = (i / columns) * frameHeight;
            frames.Add(
                new Rectangle
                {
                    x = x,
                    y = y,
                    width = frameWidth,
                    height = frameHeight
                }
            );
        }
        return frames;
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
        }

        if (animationState.Frames.Count > 1)
        {
            timeSinceLastFrame += deltaTime * animationState.SpeedScale;
            if (timeSinceLastFrame >= animationState.FrameTime)
            {
                timeSinceLastFrame = 0f;

                if (currentFrameIndex < animationState.Frames.Count - 1)
                {
                    currentFrameIndex++;
                }
                else if (animationState.ShouldLoop)
                {
                    currentFrameIndex = 0;
                }
                else
                {
                    currentFrameIndex = animationState.Frames.Count - 1; // Stay on the last frame
                }
            }
        }

        var frame = animationState.Frames[currentFrameIndex];
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
        currentFrameIndex = 0;
        timeSinceLastFrame = 0f;
    }
}

/// <summary>
/// Configuration class for animation states, supporting both grid-based and frame list-based spritesheets.
/// </summary>
public class AnimationStateConfig
{
    public string TexturePath { get; set; }
    public float FrameTime { get; set; }
    public float SpeedScale { get; set; }
    public bool ShouldLoop { get; set; }

    // Grid-based frames
    public int? FrameWidth { get; set; }
    public int? FrameHeight { get; set; }
    public int? Columns { get; set; } // Number of columns in the grid
    public int? StartFrameIndex { get; set; } // Starting frame index in the grid
    public int? FrameCount { get; set; } // Number of frames to use

    // Frame list-based frames
    public List<Rectangle> Frames { get; set; }

    // Constructor for grid-based frames with start index and frame count
    public AnimationStateConfig(
        string texturePath,
        int frameWidth,
        int frameHeight,
        float frameTime,
        float speedScale,
        bool shouldLoop,
        int startFrameIndex,
        int frameCount,
        int columns
    )
    {
        TexturePath = texturePath;
        FrameWidth = frameWidth;
        FrameHeight = frameHeight;
        FrameTime = frameTime;
        SpeedScale = speedScale;
        ShouldLoop = shouldLoop;
        StartFrameIndex = startFrameIndex;
        FrameCount = frameCount;
        Columns = columns;
    }

    // Constructor for frame list-based frames
    public AnimationStateConfig(
        string texturePath,
        List<Rectangle> frames,
        float frameTime,
        float speedScale,
        bool shouldLoop
    )
    {
        TexturePath = texturePath;
        Frames = frames;
        FrameTime = frameTime;
        SpeedScale = speedScale;
        ShouldLoop = shouldLoop;
    }
}
