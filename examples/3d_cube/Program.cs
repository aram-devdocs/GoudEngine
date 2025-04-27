using System;
using System.IO;
using System.Threading;
using CsBindgen;
using GoudEngine;

class Program
{
    static void Main(string[] args)
    {
        Console.WriteLine("Starting Basic 3D Example");

        var game = new GoudGame(800, 600, "Basic 3D Example", RendererType.Renderer3D);

        // Initialize camera controller
        var cameraController = new CameraController(game, 0, 10, -20);

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
        float lightIntensity = 2.0f;
        float lightHeight = 5.0f;
        float lightRange = 20.0f;
        float lightOrbitRadius = 8.0f;
        float lightAngle = 0.0f;
        float lightOrbitSpeed = 1.0f;

        // Sun light configuration
        uint sunLightId = 0;
        float sunLightOrbitSpeed = 0.3f;
        float sunLightOrbitRadius = 20.0f;
        float sunLightHeight = 20.0f;
        float sunLightIntensity = 50.0f;
        float sunLightRange = 50.0f;
        float sunLightAngle = 0.0f;
        float sunLightSpotAngle = 180.0f;
        float sunLightTemperature = 5500.0f;

        game.Initialize(() =>
        {
            Console.WriteLine("Game initialized");

            // Configure the grid with custom settings
            Console.WriteLine("Configuring 3D grid...");
            game.ConfigureGrid(enabled: true, renderMode: GridRenderMode.Blend);

            // Configure the skybox with custom settings
            Console.WriteLine("Configuring skybox...");
            game.ConfigureSkybox(
                enabled: true,
                size: 100.0f,
                textureSize: 128,
                rightFaceColor: new float[3] { 0.8f, 0.9f, 1.0f },
                leftFaceColor: new float[3] { 0.8f, 0.9f, 1.0f },
                topFaceColor: new float[3] { 0.8f, 0.9f, 1.0f },
                bottomFaceColor: new float[3] { 0.8f, 0.9f, 1.0f },
                frontFaceColor: new float[3] { 0.8f, 0.9f, 1.0f },
                backFaceColor: new float[3] { 0.8f, 0.9f, 1.0f },
                blendFactor: 0.5f,
                minColor: new float[3] { 0.2f, 0.3f, 0.4f },
                useCustomTextures: false
            );

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
            uint planeId = game.CreateCube(textureId, 10.0f, 1.0f, 10.0f);
            game.SetObjectPosition(planeId, 0.0f, 0.0f, 0.0f);
            game.SetObjectRotation(planeId, 0.0f, 0.0f, 0.0f);

            // Create player cube and position it at the center of the plane
            playerId = game.CreateCube(textureId, 1.0f, 1.0f, 1.0f);
            game.SetObjectPosition(playerId, 0.0f, baseHeight, 0.0f);
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

            // Add bright sun light from above
            sunLightId = game.AddLight(
                LightType.Point,
                0.0f,
                sunLightHeight,
                0.0f,
                0.0f,
                -1.0f,
                0.0f,
                1.0f,
                1.0f,
                1.0f,
                sunLightIntensity,
                sunLightTemperature,
                sunLightRange,
                sunLightSpotAngle
            );
        });

        game.Start(() =>
        {
            Console.WriteLine("Game started");
        });

        game.Update(() =>
        {
            // Update camera
            cameraController.Update(game.UpdateResponseData.delta_time);

            // Toggle grid with G key
            if (game.IsKeyPressed(71)) // G key
            {
                bool currentState = game.SetGridEnabled(true);
                game.SetGridEnabled(!currentState);
                Console.WriteLine($"Grid {(!currentState ? "enabled" : "disabled")}");
                Thread.Sleep(200);
            }

            // Toggle different grid planes with number keys
            if (game.IsKeyPressed(49)) // 1 key - Toggle XZ (floor) plane
            {
                game.SetGridPlanes(
                    showXZ: true,
                    showXY: game.IsKeyPressed(50),
                    showYZ: game.IsKeyPressed(51)
                );
                Console.WriteLine("Grid planes updated");
                Thread.Sleep(200);
            }

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

            // Update sun light position and rotation
            sunLightAngle += sunLightOrbitSpeed * game.UpdateResponseData.delta_time;

            float sunX = (float)Math.Cos(sunLightAngle) * sunLightOrbitRadius;
            float sunZ = (float)Math.Sin(sunLightAngle) * sunLightOrbitRadius;

            game.UpdateLight(
                sunLightId,
                LightType.Point,
                sunX,
                sunLightHeight,
                sunZ,
                0,
                -1,
                0,
                1.0f,
                1.0f,
                1.0f,
                sunLightIntensity,
                sunLightTemperature,
                sunLightRange,
                sunLightSpotAngle
            );
        });

        game.Terminate();
    }
}
