using System;
using System.IO;
using CsBindgen;
using GoudEngine;

class Program
{
    static void Main(string[] args)
    {
        Console.WriteLine("Starting Basic 3D Example");

        var game = new GoudGame(800, 600, "Basic 3D Example", RendererType.Renderer3D);

        // Camera state
        float cameraX = 0.0f;
        float cameraY = 10.0f;
        float cameraZ = -15.0f;
        // float cameraRotationSpeed = 0.1f;
        float cameraMoveSpeed = 0.5f;

        // Bounce animation state
        float bounceHeight = 0.5f;
        float bounceSpeed = 3.0f;
        float baseHeight = 1.5f;
        float elapsedTime = 0.0f;

        // Rotation animation state
        float rotationSpeed = 15.0f;
        float currentRotation = 0.0f;
        uint playerId = 0;

        game.Initialize(() =>
        {
            Console.WriteLine("Game initialized");

            // Initial camera position
            game.SetCameraPosition(cameraX, cameraY);
            game.SetCameraZoom(cameraZ);

            // Load texture
            string texturePath = Path.Combine(
                AppDomain.CurrentDomain.BaseDirectory,
                "assets",
                "default_grey.png"
            );

            if (!File.Exists(texturePath))
            {
                Console.WriteLine("Error: Texture file not found!");
                return;
            }

            uint textureId = game.CreateTexture(texturePath);

            // Create a simple plane
            uint planeId = game.CreateCube(textureId, 10.0f, 1.0f, 10.0f); // TODO:https://github.com/aram-devdocs/GoudEngine/issues/47 Bug: it should be x,y,z, however it appears it is x,z,y.
            game.SetObjectPosition(planeId, 0.0f, 0.0f, 0.0f);
            game.SetObjectRotation(planeId, 0.0f, 0.0f, 0.0f);

            // Create player cube and position it at the center of the plane
            playerId = game.CreateCube(textureId, 1.0f, 1.0f, 1.0f); // TODO:https://github.com/aram-devdocs/GoudEngine/issues/47 Bug: it should be x,y,z, however it appears it is x,z,y.
            game.SetObjectPosition(playerId, 0.0f, baseHeight, 0.0f); // TODO:https://github.com/aram-devdocs/GoudEngine/issues/47 Bug: it should be x,y,z, however it appears it is x,z,y.
            game.SetObjectRotation(playerId, 0.0f, 0.0f, 0.0f);
        });

        game.Start(() =>
        {
            Console.WriteLine("Game started");
        });

        game.Update(() =>
        {
            // Update bounce animation
            elapsedTime += game.UpdateResponseData.delta_time;
            float bounceOffset = (float)Math.Sin(elapsedTime * bounceSpeed) * bounceHeight;
            game.SetObjectPosition(playerId, 0.0f, baseHeight + bounceOffset, 0.0f);

            // Update rotation animation
            currentRotation += rotationSpeed * game.UpdateResponseData.delta_time;
            game.SetObjectRotation(playerId, 0.0f, currentRotation, 0.0f);

            // Camera Movement Controls
            if (game.IsKeyPressed(87)) // W key
            {
                cameraY += cameraMoveSpeed;
                game.SetCameraPosition(cameraX, cameraY);
            }
            if (game.IsKeyPressed(83)) // S key
            {
                cameraY -= cameraMoveSpeed;
                game.SetCameraPosition(cameraX, cameraY);
            }
            if (game.IsKeyPressed(65)) // A key
            {
                cameraX -= cameraMoveSpeed;
                game.SetCameraPosition(cameraX, cameraY);
            }
            if (game.IsKeyPressed(68)) // D key
            {
                cameraX += cameraMoveSpeed;
                game.SetCameraPosition(cameraX, cameraY);
            }

            // Camera Zoom Controls
            if (game.IsKeyPressed(81)) // Q key - zoom in
            {
                cameraZ += cameraMoveSpeed;
                game.SetCameraZoom(cameraZ);
            }
            if (game.IsKeyPressed(69)) // E key - zoom out
            {
                cameraZ -= cameraMoveSpeed;
                game.SetCameraZoom(cameraZ);
            }
        });

        game.Terminate();
    }
}
