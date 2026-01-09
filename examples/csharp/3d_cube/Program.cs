// 3D Cube Demo - Shows 3D cubes with lighting and camera control
//
// This example demonstrates:
// - Creating 3D primitives (cubes, planes)
// - Multiple lights with orbiting animation
// - Camera movement and rotation
// - Grid and skybox configuration

using System;
using System.IO;
using GoudEngine.Input;

class Program
{
    // Camera state
    static float positionX = 0f;
    static float positionY = 10f;
    static float positionZ = -15f;

    static float yaw = 0f;     // Rotation around Y axis (left/right)
    static float pitch = -30f; // Rotation around X axis (up/down)

    // Camera settings
    static float moveSpeed = 5f;
    static float rotationSpeed = 60f;

    // Direction vectors (calculated from yaw/pitch)
    static float[] forward = new float[3] { 0, 0, 1 };
    static float[] right = new float[3] { 1, 0, 0 };
    static float[] up = new float[3] { 0, 1, 0 };

    static void Main(string[] args)
    {
        Console.WriteLine("╔══════════════════════════════════════════╗");
        Console.WriteLine("║         3D CUBE DEMO - GoudEngine        ║");
        Console.WriteLine("╠══════════════════════════════════════════╣");
        Console.WriteLine("║  CONTROLS:                               ║");
        Console.WriteLine("║    Q / E     - Move forward / backward   ║");
        Console.WriteLine("║    A / D     - Strafe left / right       ║");
        Console.WriteLine("║    W / S     - Move up / down            ║");
        Console.WriteLine("║    ← / →     - Look left / right         ║");
        Console.WriteLine("║    ↑ / ↓     - Look up / down            ║");
        Console.WriteLine("║    G         - Toggle grid               ║");
        Console.WriteLine("║    F         - Toggle fog                ║");
        Console.WriteLine("║    ESC       - Exit                      ║");
        Console.WriteLine("╠══════════════════════════════════════════╣");
        Console.WriteLine("║  SCENE:                                  ║");
        Console.WriteLine("║    • Bouncing cube at origin             ║");
        Console.WriteLine("║    • Floor plane below                   ║");
        Console.WriteLine("║    • 3 orbiting colored lights           ║");
        Console.WriteLine("║    • Axis markers: X=5,10 Z=5,10 Y=5     ║");
        Console.WriteLine("║    • Grid shows XZ, XY, YZ planes        ║");
        Console.WriteLine("║    • RGB axis lines at origin            ║");
        Console.WriteLine("╚══════════════════════════════════════════╝");
        Console.WriteLine();

        using var game = new GoudGame(800, 600, "3D Cube Demo");

        // Configure grid (all 3 planes with axis markers at origin)
        game.ConfigureGrid(enabled: true, size: 20.0f, divisions: 20);

        // Configure skybox (moody dark atmosphere)
        game.ConfigureSkybox(enabled: true, r: 0.05f, g: 0.05f, b: 0.1f, a: 1.0f);
        
        // Configure fog for atmosphere (fade to black at distance)
        // Lower density = fog starts further away, higher = closer
        game.ConfigureFog(enabled: true, r: 0.0f, g: 0.0f, b: 0.0f, density: 0.02f);

        // Load texture
        string texturePath = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory,
            "assets",
            "default_grey.png"
        );

        uint textureId = 0;
        if (File.Exists(texturePath))
        {
            textureId = game.CreateTexture(texturePath);
            Console.WriteLine($"Loaded texture: {texturePath}");
        }
        else
        {
            Console.WriteLine($"Texture not found: {texturePath}");
        }

        // Create a large floor plane (visible below the cube)
        uint planeId = game.CreatePlane(0, 30.0f, 30.0f);  // No texture, will use default gray
        game.SetObjectPosition(planeId, 0.0f, -1.0f, 0.0f);

        // Create the main player cube (with texture)
        uint playerId = game.CreateCube(textureId, 1.0f, 1.0f, 1.0f);
        game.SetObjectPosition(playerId, 0.0f, 0.5f, 0.0f);
        
        // Create axis indicator cubes (small cubes along each axis)
        // X axis - red markers
        uint xMarker1 = game.CreateCube(0, 0.3f, 0.3f, 0.3f);
        game.SetObjectPosition(xMarker1, 5.0f, 0.0f, 0.0f);
        uint xMarker2 = game.CreateCube(0, 0.3f, 0.3f, 0.3f);
        game.SetObjectPosition(xMarker2, 10.0f, 0.0f, 0.0f);
        
        // Z axis - blue markers  
        uint zMarker1 = game.CreateCube(0, 0.3f, 0.3f, 0.3f);
        game.SetObjectPosition(zMarker1, 0.0f, 0.0f, 5.0f);
        uint zMarker2 = game.CreateCube(0, 0.3f, 0.3f, 0.3f);
        game.SetObjectPosition(zMarker2, 0.0f, 0.0f, 10.0f);
        
        // Y axis - green markers
        uint yMarker1 = game.CreateCube(0, 0.3f, 0.3f, 0.3f);
        game.SetObjectPosition(yMarker1, 0.0f, 5.0f, 0.0f);

        // Create lights
        // Red orbiting light
        uint redLightId = game.AddPointLight(5.0f, 5.0f, -5.0f, 1.0f, 0.2f, 0.2f, 2.0f, 20.0f);

        // Blue orbiting light
        uint blueLightId = game.AddPointLight(-5.0f, 5.0f, 5.0f, 0.2f, 0.2f, 1.0f, 2.0f, 20.0f);

        // Green orbiting light
        uint greenLightId = game.AddPointLight(5.0f, 5.0f, 5.0f, 0.2f, 1.0f, 0.2f, 2.0f, 20.0f);

        // Sun light (bright white from above)
        uint sunLightId = game.AddPointLight(0.0f, 20.0f, 0.0f, 1.0f, 1.0f, 0.9f, 50.0f, 50.0f);

        // Animation state
        float elapsedTime = 0.0f;
        float bounceHeight = 0.5f;
        float bounceSpeed = 3.0f;
        float baseHeight = 1.5f;
        float rotationAnimSpeed = 15.0f;
        float currentRotation = 0.0f;

        float lightOrbitRadius = 8.0f;
        float lightAngle = 0.0f;
        float lightOrbitSpeed = 1.0f;
        float lightHeight = 5.0f;

        float sunLightAngle = 0.0f;
        float sunLightOrbitRadius = 20.0f;
        float sunLightOrbitSpeed = 0.3f;

        bool gridEnabled = true;
        bool fogEnabled = true;

        // Set initial camera position
        game.SetCameraPosition3D(positionX, positionY, positionZ);
        game.SetCameraRotation(pitch, yaw, 0);

        // Main game loop
        while (!game.ShouldClose())
        {
            float deltaTime = game.DeltaTime;
            elapsedTime += deltaTime;

            // Begin frame with moody skybox color
            game.BeginFrame(0.05f, 0.05f, 0.1f, 1.0f);

            // Handle exit
            if (game.IsKeyPressed(Keys.Escape))
            {
                game.Close();
                continue;
            }

            // Handle camera input
            HandleCameraInput(game, deltaTime, ref gridEnabled, ref fogEnabled);

            // Update camera
            game.SetCameraPosition3D(positionX, positionY, positionZ);
            game.SetCameraRotation(pitch, yaw, 0);

            // Animate player cube - bounce
            float bounceOffset = (float)Math.Sin(elapsedTime * bounceSpeed) * bounceHeight;
            game.SetObjectPosition(playerId, 0.0f, baseHeight + bounceOffset, 0.0f);

            // Animate player cube - rotation
            currentRotation += rotationAnimSpeed * deltaTime;
            game.SetObjectRotation(playerId, 0.0f, currentRotation, 0.0f);

            // Animate orbiting lights
            lightAngle += lightOrbitSpeed * deltaTime;

            // Red light orbit
            float redX = (float)Math.Cos(lightAngle) * lightOrbitRadius;
            float redZ = (float)Math.Sin(lightAngle) * lightOrbitRadius;
            float redPulse = (float)(Math.Sin(lightAngle) * 0.5f + 0.5f);
            game.UpdateLight(redLightId, GoudGame.LightType.Point,
                redX, lightHeight, redZ, 0, -1, 0,
                1.0f, redPulse * 0.2f, redPulse * 0.2f, 2.0f, 20.0f, 0);

            // Blue light orbit (120 degrees offset)
            float blueX = (float)Math.Cos(lightAngle + 2.0f * Math.PI / 3.0f) * lightOrbitRadius;
            float blueZ = (float)Math.Sin(lightAngle + 2.0f * Math.PI / 3.0f) * lightOrbitRadius;
            float bluePulse = (float)(Math.Sin(lightAngle + 2.0f * Math.PI / 3.0f) * 0.5f + 0.5f);
            game.UpdateLight(blueLightId, GoudGame.LightType.Point,
                blueX, lightHeight, blueZ, 0, -1, 0,
                bluePulse * 0.2f, bluePulse * 0.2f, 1.0f, 2.0f, 20.0f, 0);

            // Green light orbit (240 degrees offset)
            float greenX = (float)Math.Cos(lightAngle + 4.0f * Math.PI / 3.0f) * lightOrbitRadius;
            float greenZ = (float)Math.Sin(lightAngle + 4.0f * Math.PI / 3.0f) * lightOrbitRadius;
            float greenPulse = (float)(Math.Sin(lightAngle + 4.0f * Math.PI / 3.0f) * 0.5f + 0.5f);
            game.UpdateLight(greenLightId, GoudGame.LightType.Point,
                greenX, lightHeight, greenZ, 0, -1, 0,
                greenPulse * 0.2f, 1.0f, greenPulse * 0.2f, 2.0f, 20.0f, 0);

            // Animate sun light
            sunLightAngle += sunLightOrbitSpeed * deltaTime;
            float sunX = (float)Math.Cos(sunLightAngle) * sunLightOrbitRadius;
            float sunZ = (float)Math.Sin(sunLightAngle) * sunLightOrbitRadius;
            game.UpdateLight(sunLightId, GoudGame.LightType.Point,
                sunX, 20.0f, sunZ, 0, -1, 0,
                1.0f, 1.0f, 0.9f, 50.0f, 50.0f, 0);

            // Render 3D scene
            game.Render3D();

            // End frame
            game.EndFrame();
        }

        Console.WriteLine("3D Demo ended");
    }

    static void HandleCameraInput(GoudGame game, float deltaTime, ref bool gridEnabled, ref bool fogEnabled)
    {
        bool moved = false;
        UpdateDirectionVectors();

        float moveDelta = moveSpeed * deltaTime;

        // Forward/Backward movement using Q/E
        if (game.IsKeyPressed(Keys.Q))
        {
            positionX += forward[0] * moveDelta;
            positionY += forward[1] * moveDelta;
            positionZ += forward[2] * moveDelta;
            moved = true;
        }
        if (game.IsKeyPressed(Keys.E))
        {
            positionX -= forward[0] * moveDelta;
            positionY -= forward[1] * moveDelta;
            positionZ -= forward[2] * moveDelta;
            moved = true;
        }

        // Strafe movement using A/D (fixed: A = left, D = right)
        if (game.IsKeyPressed(Keys.A))
        {
            positionX += right[0] * moveDelta;
            positionZ += right[2] * moveDelta;
            moved = true;
        }
        if (game.IsKeyPressed(Keys.D))
        {
            positionX -= right[0] * moveDelta;
            positionZ -= right[2] * moveDelta;
            moved = true;
        }

        // Up/Down movement using W/S
        if (game.IsKeyPressed(Keys.W))
        {
            positionY += moveDelta;
            moved = true;
        }
        if (game.IsKeyPressed(Keys.S))
        {
            positionY -= moveDelta;
            moved = true;
        }

        // Keep minimum height
        positionY = Math.Max(positionY, 0.5f);

        // Handle rotation using arrow keys (fixed: Left = look left, Right = look right)
        if (game.IsKeyPressed(Keys.Left))
        {
            yaw += rotationSpeed * deltaTime;
            moved = true;
        }
        if (game.IsKeyPressed(Keys.Right))
        {
            yaw -= rotationSpeed * deltaTime;
            moved = true;
        }
        if (game.IsKeyPressed(Keys.Up))
        {
            pitch += rotationSpeed * deltaTime;
            pitch = Math.Clamp(pitch, -89.0f, 89.0f);
            moved = true;
        }
        if (game.IsKeyPressed(Keys.Down))
        {
            pitch -= rotationSpeed * deltaTime;
            pitch = Math.Clamp(pitch, -89.0f, 89.0f);
            moved = true;
        }

        // Toggle grid with G key
        if (game.IsKeyJustPressed(Keys.G))
        {
            gridEnabled = !gridEnabled;
            game.SetGridEnabled(gridEnabled);
            Console.WriteLine($"Grid {(gridEnabled ? "enabled" : "disabled")}");
        }
        
        // Toggle fog with F key
        if (game.IsKeyJustPressed(Keys.F))
        {
            fogEnabled = !fogEnabled;
            game.SetFogEnabled(fogEnabled);
            Console.WriteLine($"Fog {(fogEnabled ? "enabled" : "disabled")}");
        }

        if (moved)
        {
            UpdateDirectionVectors();
        }
    }

    static void UpdateDirectionVectors()
    {
        float yawRad = yaw * (float)Math.PI / 180.0f;
        float pitchRad = pitch * (float)Math.PI / 180.0f;

        // Calculate new forward vector
        forward[0] = (float)(Math.Sin(yawRad) * Math.Cos(pitchRad));
        forward[1] = (float)Math.Sin(pitchRad);
        forward[2] = (float)(Math.Cos(yawRad) * Math.Cos(pitchRad));

        // Calculate right vector (perpendicular to forward on XZ plane)
        right[0] = (float)Math.Cos(yawRad);
        right[1] = 0;
        right[2] = (float)-Math.Sin(yawRad);
    }
}
