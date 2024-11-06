using System;
using System.Collections.Generic;

public class GameManager
{
    private GoudGame game;
    private Bird bird;
    private List<PipePair> pipes = new List<PipePair>();
    private int score = 0;

    private float pipeSpawnTimer = 0.0f;

    private Movement movement;

    public GameManager(GoudGame game)
    {
        this.game = game;
        bird = new Bird(game, 400);
        movement = new Movement(game);
        movement.AddSprite(bird.birdSpriteIndex, bird.data);
        Console.WriteLine("Game Manager Initialized!");


    }

    public void Update()
    {
        bird.Update();
        movement.Update();

        // Spawn pipes every 2 seconds
        pipeSpawnTimer += GameConstants.PipeSpeed;
        if (pipeSpawnTimer > 200)
        {
            pipes.Add(new PipePair(game, 800)); // Start from the right
            pipeSpawnTimer = 0;
        }

        // Update pipes and check for collisions
        foreach (var pipe in pipes)
        {
            pipe.Update();

            // Check if the bird passed the pipe
            if (pipe.GetXPosition() < 90 && !pipe.CheckCollision(bird.GetYPosition()))
            {
                score++;
                Console.WriteLine("Score: " + score);
            }


            // TODO: https://github.com/aram-devdocs/GoudEngine/issues/3

            // Check for collision
            // if (pipe.CheckCollision(bird.GetYPosition()))
            // {
            //     EndGame();
            //     return;
            // }
        }

        // Remove offscreen pipes
        pipes.RemoveAll(pipe => pipe.IsOffScreen());


        // TODO: https://github.com/aram-devdocs/GoudEngine/issues/3

        // Check if bird hit the ground
        // if (bird.HasHitGround())
        // {
        //     EndGame();
        // }
    }

    private void EndGame()
    {
        Console.WriteLine("Game Over!");
        Console.WriteLine("Final Score: " + score);
        game.Terminate();
    }
}