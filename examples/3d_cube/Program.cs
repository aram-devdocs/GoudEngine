using System;
using System.IO;
using GoudEngine;
using CsBindgen;

class Program
{
    private static float rotationAngle = 0.0f;
    private static uint cubeId;
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

            // Set initial camera position and zoom
            game.SetCameraPosition(0.0f, 0.0f);
            game.SetCameraZoom(5.0f);

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

            // Create a cube
            cubeId = game.CreateCube(textureId);
            Console.WriteLine($"Cube created with ID: {cubeId}");

            // Set initial cube position
            game.SetObjectPosition(cubeId, 0.0f, 0.0f, 0.0f);
            game.SetObjectScale(cubeId, 1.0f, 1.0f, 1.0f);
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

            // Rotate the cube
            game.SetObjectRotation(cubeId, rotationAngle, rotationAngle, 0.0f);

            // Make the cube "breathe" by scaling it up and down
            float scale = 1.0f + (float)Math.Sin(rotationAngle * Math.PI / 180.0f) * 0.2f;
            game.SetObjectScale(cubeId, scale, scale, scale);

            // Move camera in a circle around the cube
            float cameraRadius = 3.0f;
            float cameraX = (float)Math.Sin(rotationAngle * Math.PI / 180.0f) * cameraRadius;
            float cameraY = (float)Math.Cos(rotationAngle * Math.PI / 180.0f) * cameraRadius;
            game.SetCameraPosition(cameraX, cameraY);
        });

        // Clean up
        game.Terminate();
        Console.WriteLine("Game terminated");
    }
}
