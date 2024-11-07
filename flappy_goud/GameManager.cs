// GameManager.cs

using System;
using System.Collections.Generic;
using CsBindgen;

public class GameManager
{
    private GoudGame game;
    private Bird bird;
    private List<Pipe> pipes;
    private int score;
    private float pipeSpawnTimer;

    public GameManager(GoudGame game)
    {
        this.game = game;
        this.bird = new Bird(game);
        this.pipes = new List<Pipe>();
        this.score = 0;
        this.pipeSpawnTimer = 0;
    }

    public void Initialize()
    {
        SpriteData backgroundData = new SpriteData
        {
            x = 0,
            y = 0,

            // TODO: Passing values here breaks the game
            // dimmension_x = 800, // Set to window width
            // dimmension_y = 600, // Set to window height
        };
        // Background
        game.AddSprite("assets/sprites/background-day.png", backgroundData);
        bird.Initialize();

    }

    public void Start()
    {
        bird.Reset();
        pipes.Clear();
        score = 0;
        pipeSpawnTimer = 0;
    }

    public void Update()
    {

        // If R is pressed, restart the game
        if (game.IsKeyPressed(82))
        {
            ResetGame();
            return;
        }


        // Update Bird Movement
        bird.Update();

        // Check for collisions
        foreach (var pipe in pipes)
        {
            pipe.Update();
            if (bird.CollidesWith(pipe))
            {
                ResetGame();
                return;
            }
        }

        // Spawn new pipes
        pipeSpawnTimer += GameConstants.DeltaTime;
        if (pipeSpawnTimer > GameConstants.PipeSpawnInterval)
        {
            pipeSpawnTimer = 0;
            pipes.Add(new Pipe(game));
        }

        // Remove off-screen pipes
        pipes.RemoveAll(pipe => pipe.IsOffScreen());

        // Update Score
        score += bird.PassedPipes(pipes);
    }

    private void ResetGame()
    {
        Start(); // Restart the game
    }
}