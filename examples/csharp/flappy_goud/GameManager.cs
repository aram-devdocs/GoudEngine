// GameManager.cs

using System;
using System.Collections.Generic;
using GoudEngine.Input;

public class GameManager
{
    private GoudGame game;

    private ScoreCounter scoreCounter;
    private Bird bird;
    private List<Pipe> pipes;
    private float pipeSpawnTimer;

    // Texture IDs for immediate-mode rendering (64-bit handles)
    private ulong backgroundTextureId;
    private ulong baseTextureId;
    private ulong pipeTextureId;

    // Background dimensions (actual image: 288 × 512)
    private const float BackgroundWidth = 288f;
    private const float BackgroundHeight = 512f;

    // Base dimensions (actual image: 336 × 112)
    private const float BaseWidth = 336f;
    private const float BaseHeight = 112f;

    public GameManager(GoudGame game)
    {
        this.game = game;
        bird = new Bird(game);
        pipes = new List<Pipe>();
        pipeSpawnTimer = 0;
        scoreCounter = new ScoreCounter();
    }

    public void Initialize()
    {
        // Load all textures once at initialization (64-bit handles)
        backgroundTextureId = game.LoadTexture("assets/sprites/background-day.png");
        baseTextureId = game.LoadTexture("assets/sprites/base.png");
        pipeTextureId = game.LoadTexture("assets/sprites/pipe-green.png");

        bird.Initialize();
        scoreCounter.Initialize(game);
    }

    public void Start()
    {
        bird.Reset();
        pipes.Clear();
        scoreCounter.ResetScore();
        pipeSpawnTimer = 0;
    }

    public void Update(float deltaTime)
    {
        // If Escape is pressed, close the game
        if (game.IsKeyPressed(Keys.Escape))
        {
            game.Close();
            return;
        }

        // If R is pressed, restart the game
        if (game.IsKeyPressed(Keys.R))
        {
            ResetGame();
            return;
        }

        // Update Bird Movement with deltaTime
        bird.Update(deltaTime);

        // Check if bird is colliding with the base (ground)
        if (bird.Y + Bird.Height > GameConstants.ScreenHeight)
        {
            ResetGame();
            return;
        }

        // Check if bird is off-screen (top)
        if (bird.Y < 0)
        {
            ResetGame();
            return;
        }

        // Update Pipes with deltaTime and check for collisions
        foreach (var pipe in pipes)
        {
            pipe.Update(deltaTime);

            // Use AABB collision detection
            if (pipe.CollidesWithBird(bird.X, bird.Y, Bird.Width, Bird.Height))
            {
                ResetGame();
                return;
            }
        }

        // Spawn new pipes
        pipeSpawnTimer += deltaTime;
        if (pipeSpawnTimer > GameConstants.PipeSpawnInterval)
        {
            pipeSpawnTimer = 0;
            pipes.Add(new Pipe(game));
        }

        // Remove off-screen pipes and increment score
        pipes.RemoveAll(pipe =>
        {
            if (pipe.IsOffScreen())
            {
                scoreCounter.IncrementScore();
                return true;
            }
            return false;
        });

        // Draw everything (immediate-mode rendering)
        Draw();
    }

    /// <summary>
    /// Draws all game objects using immediate-mode rendering.
    /// Called each frame after Update.
    /// Note: DrawSprite uses center-based positioning, so we offset by half width/height.
    /// Draw order determines layering (later = on top).
    /// </summary>
    private void Draw()
    {
        // Layer 0: Background
        game.DrawSprite(backgroundTextureId, 
            BackgroundWidth / 2, 
            BackgroundHeight / 2, 
            BackgroundWidth, BackgroundHeight);

        // Layer 1: Score (behind pipes, in front of background)
        scoreCounter.Draw(game);

        // Layer 2: Pipes
        foreach (var pipe in pipes)
        {
            pipe.Draw(pipeTextureId);
        }

        // Layer 3: Bird (in front of pipes)
        bird.Draw();

        // Layer 4: Base/ground (in front of everything)
        game.DrawSprite(baseTextureId, 
            BaseWidth / 2, 
            GameConstants.ScreenHeight + BaseHeight / 2, 
            BaseWidth, BaseHeight);
    }

    private void ResetGame()
    {
        scoreCounter.ResetScore();
        Start(); // Restart the game
    }
}
