using System;
using System.IO;
using CsBindgen;
using GoudEngine;

class Program
{
    private static float rotationAngle = 0.0f;
    private static uint[] objectIds = new uint[4]; // Store IDs for cube, sphere, cylinder, and plane
    private static uint textureId;

    static void Main(string[] args)
    {
        Console.WriteLine("Starting 3D Primitives Example");

        // Create a game instance with 3D renderer
        var game = new GoudGame(800, 600, "3D Primitives Example", RendererType.Renderer3D);

        // Initialize the game
        game.Initialize(() =>
        {
            Console.WriteLine("Game initialized");

            // Set initial camera position and zoom
            game.SetCameraPosition(0.0f, 0.0f);
            game.SetCameraZoom(10.0f);

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

            // Create primitives using convenience methods
            objectIds[0] = game.CreateCube(textureId, 1.0f, 1.0f, 1.0f);
            Console.WriteLine($"Cube created with ID: {objectIds[0]}");

            objectIds[1] = game.CreateSphere(textureId, 0.5f, 32);
            Console.WriteLine($"Sphere created with ID: {objectIds[1]}");

            objectIds[2] = game.CreateCylinder(textureId, 0.5f, 2.0f, 32);
            Console.WriteLine($"Cylinder created with ID: {objectIds[2]}");

            objectIds[3] = game.CreatePlane(textureId, 5.0f, 5.0f);
            Console.WriteLine($"Plane created with ID: {objectIds[3]}");

            // Position objects
            game.SetObjectPosition(objectIds[0], -2.0f, 0.0f, 0.0f); // Cube
            game.SetObjectPosition(objectIds[1], 2.0f, 0.0f, 0.0f); // Sphere
            game.SetObjectPosition(objectIds[2], 0.0f, 2.0f, 0.0f); // Cylinder
            game.SetObjectPosition(objectIds[3], 0.0f, -2.0f, 0.0f); // Plane

            // Initial scales
            game.SetObjectScale(objectIds[0], 1.0f, 1.0f, 1.0f);
            game.SetObjectScale(objectIds[1], 1.0f, 1.0f, 1.0f);
            game.SetObjectScale(objectIds[2], 1.0f, 1.0f, 1.0f);
            game.SetObjectScale(objectIds[3], 1.0f, 1.0f, 1.0f);
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

            // Rotate objects differently
            game.SetObjectRotation(objectIds[0], rotationAngle, rotationAngle, 0.0f); // Cube rotates on X and Y
            game.SetObjectRotation(objectIds[1], 0.0f, rotationAngle, rotationAngle); // Sphere rotates on Y and Z
            game.SetObjectRotation(objectIds[2], rotationAngle, 0.0f, rotationAngle); // Cylinder rotates on X and Z
            game.SetObjectRotation(objectIds[3], rotationAngle * 0.25f, 0.0f, 0.0f); // Plane slowly rotates on X

            // Make objects "breathe" with different phases
            float baseScale = 1.0f + (float)Math.Sin(rotationAngle * Math.PI / 180.0f) * 0.2f;
            game.SetObjectScale(objectIds[0], baseScale, baseScale, baseScale);

            float sphereScale = 1.0f + (float)Math.Cos(rotationAngle * Math.PI / 180.0f) * 0.2f;
            game.SetObjectScale(objectIds[1], sphereScale, sphereScale, sphereScale);

            float cylinderScale =
                1.0f + (float)Math.Sin((rotationAngle + 90.0f) * Math.PI / 180.0f) * 0.2f;
            game.SetObjectScale(objectIds[2], cylinderScale, cylinderScale, cylinderScale);

            // Move camera in a circle around the scene
            float cameraRadius = 8.0f;
            float cameraX = (float)Math.Sin(rotationAngle * Math.PI / 180.0f) * cameraRadius;
            float cameraY = (float)Math.Cos(rotationAngle * Math.PI / 180.0f) * cameraRadius;
            game.SetCameraPosition(cameraX, cameraY);
        });

        // Clean up
        game.Terminate();
        Console.WriteLine("Game terminated");
    }
}
