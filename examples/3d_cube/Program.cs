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

        // Light state
        uint redLightId = 0;
        uint blueLightId = 0;
        uint greenLightId = 0;
        uint spotlightId = 0;
        float lightIntensity = 2.0f;
        float lightRange = 20.0f;
        float lightOrbitRadius = 8.0f;
        float lightHeight = 5.0f;
        float lightAngle = 0.0f;
        float lightOrbitSpeed = 1.0f;

        game.Initialize(() =>
        {
            Console.WriteLine("Game initialized");

            // Enable debug mode first, before creating any objects
            game.SetDebugMode(true);

            // Set initial camera position for better grid visibility
            cameraX = 0.0f;
            cameraY = 10.0f;
            cameraZ = -15.0f;
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

            // Create three orbiting lights with different phase angles
            redLightId = game.AddLight(
                LightType.Point,
                5.0f,
                5.0f,
                -5.0f,
                0,
                -1,
                0,
                1.0f,
                0.2f,
                0.2f,
                lightIntensity,
                6500.0f,
                lightRange,
                45.0f
            );

            blueLightId = game.AddLight(
                LightType.Point,
                -5.0f,
                5.0f,
                5.0f,
                0,
                -1,
                0,
                0.2f,
                0.2f,
                1.0f,
                lightIntensity,
                6500.0f,
                lightRange,
                45.0f
            );

            greenLightId = game.AddLight(
                LightType.Point,
                5.0f,
                5.0f,
                5.0f,
                0,
                -1,
                0,
                0.2f,
                1.0f,
                0.2f,
                lightIntensity,
                6500.0f,
                lightRange,
                45.0f
            );

            // Add white spotlight from above
            spotlightId = game.AddLight(
                LightType.Spot,
                0.0f,
                10.0f,
                0.0f, // Position directly above
                0.0f,
                -1.0f,
                0.0f, // Direction pointing straight down
                1.0f,
                1.0f,
                1.0f, // White color
                3.0f, // Intensity increased from 1.5 to 3.0
                6500.0f, // Temperature
                15.0f, // Range
                60.0f // Spot angle increased from 45 to 60 degrees for wider spread
            );
        });

        game.Start(() =>
        {
            Console.WriteLine("Game started");
        });

        game.Update(() =>
        {
            // Update light positions and colors
            lightAngle += lightOrbitSpeed * game.UpdateResponseData.delta_time;

            // Calculate positions with phase offsets
            float redX = (float)Math.Cos(lightAngle) * lightOrbitRadius;
            float redZ = (float)Math.Sin(lightAngle) * lightOrbitRadius;

            float blueX = (float)Math.Cos(lightAngle + 2.0f * Math.PI / 3.0f) * lightOrbitRadius;
            float blueZ = (float)Math.Sin(lightAngle + 2.0f * Math.PI / 3.0f) * lightOrbitRadius;

            float greenX = (float)Math.Cos(lightAngle + 4.0f * Math.PI / 3.0f) * lightOrbitRadius;
            float greenZ = (float)Math.Sin(lightAngle + 4.0f * Math.PI / 3.0f) * lightOrbitRadius;

            // Pulsing colors based on position
            float redPulse = (float)(Math.Sin(lightAngle) * 0.5f + 0.5f);
            float bluePulse = (float)(Math.Sin(lightAngle + 2.0f * Math.PI / 3.0f) * 0.5f + 0.5f);
            float greenPulse = (float)(Math.Sin(lightAngle + 4.0f * Math.PI / 3.0f) * 0.5f + 0.5f);

            game.UpdateLight(
                redLightId,
                LightType.Point,
                redX,
                lightHeight,
                redZ,
                0,
                0,
                0,
                1.0f,
                redPulse * 0.2f,
                redPulse * 0.2f,
                lightIntensity,
                6500.0f,
                lightRange,
                0.0f
            );

            game.UpdateLight(
                blueLightId,
                LightType.Point,
                blueX,
                lightHeight,
                blueZ,
                0,
                0,
                0,
                bluePulse * 0.2f,
                bluePulse * 0.2f,
                1.0f,
                lightIntensity,
                6500.0f,
                lightRange,
                0.0f
            );

            game.UpdateLight(
                greenLightId,
                LightType.Point,
                greenX,
                lightHeight,
                greenZ,
                0,
                0,
                0,
                greenPulse * 0.2f,
                1.0f,
                greenPulse * 0.2f,
                lightIntensity,
                6500.0f,
                lightRange,
                0.0f
            );

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
            if (game.IsKeyPressed(68)) // D key
            {
                cameraX -= cameraMoveSpeed;
                game.SetCameraPosition(cameraX, cameraY);
            }
            if (game.IsKeyPressed(65)) // A key
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
