using System;
using System.IO;
using GoudEngine;

class Program
{
    private static float rotationAngle = 0.0f;
    private static uint textureId;

    static void Main(string[] args)
    {
        Console.WriteLine("Starting 3D Cube Example");

        // Create a game instance with 3D renderer
        var game = new GoudGame(800, 600, "3D Cube Example", RendererType.Renderer3D);

        // Initialize the game
        game.Initialize(() =>
        {
            Console.WriteLine("Game initialized");

            // Set initial camera position
            game.SetCameraPosition(0.0f, 0.0f);
            game.SetCameraZoom(5.0f); // Set camera back to see the cube better

            // Load texture
            string texturePath = Path.Combine(
                AppDomain.CurrentDomain.BaseDirectory,
                "assets",
                "default_grey.png"
            );
            Console.WriteLine($"Loading texture from: {texturePath}");

            if (!File.Exists(texturePath))
            {
                Console.WriteLine("Error: Texture file not found!");
                return;
            }

            textureId = game.CreateTexture(texturePath);
            Console.WriteLine($"Texture loaded with ID: {textureId}");
        });

        // Start the game
        game.Start(() =>
        {
            Console.WriteLine("Game started");
        });

        // Main game loop
        game.Update(() =>
        {
            // Update rotation angle
            rotationAngle += game.UpdateResponseData.delta_time * 45.0f; // 45 degrees per second
            if (rotationAngle >= 360.0f)
            {
                rotationAngle -= 360.0f;
            }

            // Calculate camera position for a circular orbit
            float radius = 2.0f;
            float cameraX = (float)Math.Sin(rotationAngle * Math.PI / 180.0f) * radius;
            float cameraY = (float)Math.Cos(rotationAngle * Math.PI / 180.0f) * radius;

            // Update camera position
            game.SetCameraPosition(cameraX, cameraY);
        });

        // Clean up
        game.Terminate();
        Console.WriteLine("Game terminated");
    }
}
