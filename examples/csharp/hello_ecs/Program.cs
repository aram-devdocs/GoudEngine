// hello_ecs - A simple example demonstrating the GoudEngine ECS and Window API
//
// This example shows how to:
// 1. Create a game window with GoudGame
// 2. Handle keyboard and mouse input
// 3. Use the immediate-mode game loop pattern
// 4. Draw quads and sprites

using System;
using GoudEngine.Input;
using GoudEngine.Math;

public class Program
{
    // Screen dimensions
    const uint ScreenWidth = 800;
    const uint ScreenHeight = 600;
    
    // Player state
    static float playerX = 400f;
    static float playerY = 300f;
    static float playerSpeed = 200f;
    static float playerSize = 40f;

    // Target state (for collision demo)
    static float targetX = 600f;
    static float targetY = 300f;
    static float targetSize = 60f;

    static void Main(string[] args)
    {
        Console.WriteLine("Hello ECS - GoudEngine Demo");
        Console.WriteLine("Controls: WASD to move, ESC to exit");
        Console.WriteLine("Move the player (blue) to collide with the target (red)!");
        Console.WriteLine();

        using var game = new GoudGame(ScreenWidth, ScreenHeight, "Hello ECS - GoudEngine");

        // Main game loop
        while (!game.ShouldClose())
        {
            // Begin frame (polls events, clears screen)
            bool isColliding = CheckCollision();
            
            // Clear with different color based on collision
            if (isColliding)
            {
                game.BeginFrame(0.2f, 0.4f, 0.2f, 1.0f); // Green when colliding
            }
            else
            {
                game.BeginFrame(0.1f, 0.1f, 0.2f, 1.0f); // Dark blue normally
            }

            float deltaTime = game.DeltaTime;

            // Check for exit
            if (game.IsKeyPressed(Keys.Escape))
            {
                game.Close();
                continue;
            }

            // Update player
            Update(game, deltaTime);

            // Render
            Render(game, isColliding);

            // End frame (swaps buffers)
            game.EndFrame();
        }

        Console.WriteLine("Game exited cleanly.");
    }

    static void Update(GoudGame game, float deltaTime)
    {
        float dx = 0, dy = 0;

        // Handle input
        if (game.IsKeyPressed(Keys.W) || game.IsKeyPressed(Keys.Up))
            dy -= 1;
        if (game.IsKeyPressed(Keys.S) || game.IsKeyPressed(Keys.Down))
            dy += 1;
        if (game.IsKeyPressed(Keys.A) || game.IsKeyPressed(Keys.Left))
            dx -= 1;
        if (game.IsKeyPressed(Keys.D) || game.IsKeyPressed(Keys.Right))
            dx += 1;

        // Normalize diagonal movement
        if (dx != 0 && dy != 0)
        {
            float length = MathF.Sqrt(dx * dx + dy * dy);
            dx /= length;
            dy /= length;
        }

        // Apply movement
        playerX += dx * playerSpeed * deltaTime;
        playerY += dy * playerSpeed * deltaTime;

        // Clamp to screen bounds
        playerX = Math.Clamp(playerX, playerSize / 2, ScreenWidth - playerSize / 2);
        playerY = Math.Clamp(playerY, playerSize / 2, ScreenHeight - playerSize / 2);
    }

    static bool CheckCollision()
    {
        // Simple AABB collision
        float left1 = playerX - playerSize / 2;
        float right1 = playerX + playerSize / 2;
        float top1 = playerY - playerSize / 2;
        float bottom1 = playerY + playerSize / 2;

        float left2 = targetX - targetSize / 2;
        float right2 = targetX + targetSize / 2;
        float top2 = targetY - targetSize / 2;
        float bottom2 = targetY + targetSize / 2;

        return left1 < right2 && right1 > left2 && top1 < bottom2 && bottom1 > top2;
    }

    static void Render(GoudGame game, bool isColliding)
    {
        // Draw target (red when not colliding, yellow when colliding)
        Color targetColor = isColliding 
            ? new Color(1.0f, 1.0f, 0.2f, 1.0f)  // Yellow
            : new Color(0.8f, 0.2f, 0.2f, 1.0f); // Red
        game.DrawQuad(targetX, targetY, targetSize, targetSize, targetColor);

        // Draw player (blue)
        game.DrawQuad(playerX, playerY, playerSize, playerSize, 
            new Color(0.2f, 0.4f, 0.9f, 1.0f));
        
        // Draw instructions at bottom
        float instructionY = ScreenHeight - 20;
        for (int i = 0; i < 5; i++)
        {
            // Draw small dots to indicate "move to target"
            game.DrawQuad(380 + i * 10, instructionY, 4, 4, 
                new Color(0.5f, 0.5f, 0.5f, 0.5f));
        }
    }
}
