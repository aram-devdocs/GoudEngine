// GameManager.cs

using System;
using System.Collections.Generic;
using CsBindgen;

public class GameManager
{
    private GoudGame game;

    private ScoreCounter scoreCounter;
    private Bird bird;
    private List<Pipe> pipes;
    private float pipeSpawnTimer;

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
        uint TextureId = game.CreateTexture("assets/sprites/background-day.png");
        SpriteCreateDto backgroundData = new SpriteCreateDto
        {
            x = 0,
            y = 0,
            texture_id = TextureId
        };

        game.AddSprite(backgroundData);
        bird.Initialize();
        scoreCounter.Initialize(game);
    }

    public void Start()
    {
        bird.Reset();
        pipes.ForEach(pipe => pipe.Remove());
        pipes.Clear();

        scoreCounter.ResetScore(game);
        pipeSpawnTimer = 0;
    }

    public void Update(float deltaTime)

    {


        // If Escape is pressed, close the game
        if (game.IsKeyPressed(256))
        {
            game.Close();
            return;
        }


        // If R is pressed, restart the game
        if (game.IsKeyPressed(82))
        {
            ResetGame();
            return;
        }

        // Update Bird Movement with deltaTime
        bird.Update(deltaTime);


        // Check if bird is off-screen
        if (bird.Y < 0 || bird.Y > GameConstants.ScreenHeight)
        {
            ResetGame();
            return;
        }

        // Update Pipes with deltaTime and check for collisions
        foreach (var pipe in pipes)
        {


            pipe.Update(deltaTime);

            if (game.CheckCollision(bird.GetSpriteId(), pipe.topSpriteId) || game.CheckCollision(bird.GetSpriteId(), pipe.bottomSpriteId))
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

        // Remove off-screen pipes
        pipes.RemoveAll(pipe =>
        {
            if (pipe.IsOffScreen())
            {
                pipe.Remove();
                scoreCounter.IncrementScore(game);
                return true;
            }
            return false;
        });


        // Update Score
        scoreCounter.Update(game);


    }

    private void ResetGame()
    {
        Start(); // Restart the game
    }
}