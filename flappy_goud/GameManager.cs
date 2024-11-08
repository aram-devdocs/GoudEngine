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
        SpriteDto backgroundData = new SpriteDto
        {
            x = 0,
            y = 0,
        };

        game.AddSprite("assets/sprites/background-day.png", backgroundData);
        bird.Initialize();
    }

    public void Start()
    {
        bird.Reset();
        pipes.ForEach(pipe => pipe.Remove());
        pipes.Clear();

        // score = 0;
        // pipeSpawnTimer = 0;
    }

    public void Update(float deltaTime)
    {
        // If R is pressed, restart the game
        if (game.IsKeyPressed(82))
        {
            ResetGame();
            return;
        }

        // Update Bird Movement with deltaTime
        bird.Update(deltaTime);

        // Update Pipes with deltaTime and check for collisions
        foreach (var pipe in pipes)
        {
            pipe.Update(deltaTime);
            // if (bird.CollidesWith(pipe))
            // {
            //     ResetGame();
            //     return;
            // }
        }

        // Spawn new pipes
        pipeSpawnTimer += deltaTime;
        if (pipeSpawnTimer > GameConstants.PipeSpawnInterval)
        {
            pipeSpawnTimer = 0;
            pipes.Add(new Pipe(game));
        }

        // Remove off-screen pipes
        pipes.RemoveAll(pipe =>
        {
            if (pipe.IsOffScreen())
            {
                pipe.Remove();
                return true;
            }
            return false;
        });

        // Update Score
        score += bird.PassedPipes(pipes);
    }

    private void ResetGame()
    {
        Start(); // Restart the game
    }
}