// hello_ecs - A simple example demonstrating the new ECS Window FFI API
//
// This example shows how to:
// 1. Create a window with GoudWindow
// 2. Handle input with GoudInput
// 3. Use the game loop pattern
// 4. Check collision with GoudCollision

using System;
using GoudEngine.Core;

public class Program
{
    // Player state
    static float playerX = 400f;
    static float playerY = 300f;
    static float playerSpeed = 200f;

    // Target state (for collision demo)
    static float targetX = 600f;
    static float targetY = 300f;
    static float targetRadius = 30f;
    static bool isColliding = false;

    static void Main(string[] args)
    {
        Console.WriteLine("Hello ECS - GoudEngine Window FFI Demo");
        Console.WriteLine("Controls: WASD to move, ESC to exit");
        Console.WriteLine();

        // Create window using new GoudWindow API
        using var window = new GoudWindow(800, 600, "Hello ECS - GoudEngine");
        
        // Create input handler
        var input = new GoudInput(window);
        
        // Create renderer
        var renderer = new GoudRenderer(window);
        
        // Setup action mappings
        input.MapAction("MoveUp", KeyCode.W);
        input.MapAction("MoveDown", KeyCode.S);
        input.MapAction("MoveLeft", KeyCode.A);
        input.MapAction("MoveRight", KeyCode.D);

        Console.WriteLine($"Window created: {window}");

        // Main game loop
        while (!window.ShouldClose)
        {
            // Poll events and get delta time
            float deltaTime = window.PollEvents();

            // Check for exit
            if (input.IsKeyPressed(KeyCode.Escape))
            {
                window.SetShouldClose(true);
                continue;
            }

            // Update player
            Update(input, deltaTime);

            // Check collision
            CheckCollision();

            // Render
            Render(window, renderer);
        }

        Console.WriteLine("Game exited cleanly.");
    }

    static void Update(GoudInput input, float deltaTime)
    {
        float dx = 0, dy = 0;

        // Handle input using actions
        if (input.IsActionPressed("MoveUp"))
            dy -= 1;
        if (input.IsActionPressed("MoveDown"))
            dy += 1;
        if (input.IsActionPressed("MoveLeft"))
            dx -= 1;
        if (input.IsActionPressed("MoveRight"))
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
        playerX = Math.Clamp(playerX, 0, 800);
        playerY = Math.Clamp(playerY, 0, 600);
    }

    static void CheckCollision()
    {
        // Use the collision utility from GoudCollision
        float distance = GoudCollision.Distance(playerX, playerY, targetX, targetY);
        float playerRadius = 20f;
        
        isColliding = distance < (playerRadius + targetRadius);
    }

    static void Render(GoudWindow window, GoudRenderer renderer)
    {
        // Clear with different color based on collision
        if (isColliding)
        {
            window.Clear(0.3f, 0.5f, 0.3f, 1.0f); // Green when colliding
        }
        else
        {
            window.Clear(0.1f, 0.1f, 0.2f, 1.0f); // Dark blue normally
        }

        // Begin rendering
        renderer.Begin();
        renderer.EnableBlending();

        // Note: For actual sprite rendering, you would load textures and draw them
        // This example just demonstrates the window/input/collision APIs

        // End rendering
        renderer.End();

        // Swap buffers
        window.SwapBuffers();
    }
}
