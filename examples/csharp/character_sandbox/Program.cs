// Character Sandbox - Demonstrates model loading, skeletal animation,
// scene management, character movement, and third-person camera.
//
// This example loads a glTF (.glb) animated model (the Khronos Fox sample),
// instantiates several NPCs, and lets the player walk around a lit ground
// plane with smooth animation transitions.
//
// CONTROLS:
//   W / S        - Move forward / backward
//   A / D        - Strafe left / right
//   Left Shift   - Hold to run
//   Arrow keys   - Rotate camera yaw (left/right) and pitch (up/down)
//   G            - Toggle debug grid
//   F            - Toggle fog
//   ESC          - Quit

using System;
using System.IO;
using GoudEngine;

class Program
{
    // ----------------------------------------------------------------
    // Animation state machine
    // ----------------------------------------------------------------
    enum AnimState { Idle, Walk, Run }

    // ----------------------------------------------------------------
    // Camera parameters (third-person)
    // ----------------------------------------------------------------
    static float cameraPitch = 25f;   // degrees above horizontal
    static float cameraYaw   = 0f;    // degrees around Y axis
    static float cameraDistance = 12f; // distance behind character
    static float cameraHeight  = 6f;  // height above character

    static float cameraRotSpeed = 90f; // degrees per second

    // ----------------------------------------------------------------
    // Movement parameters
    // ----------------------------------------------------------------
    static float walkSpeed = 4f;
    static float runSpeed  = 9f;

    // ----------------------------------------------------------------
    // Player state
    // ----------------------------------------------------------------
    static float playerX = 0f;
    static float playerY = 0f;
    static float playerZ = 0f;
    static float playerFacing = 0f; // degrees around Y

    static AnimState currentAnim = AnimState.Idle;
    static float animTransitionTime = 0.25f; // seconds

    // ----------------------------------------------------------------
    // Entry point
    // ----------------------------------------------------------------
    static void Main(string[] args)
    {
        Console.WriteLine("========================================");
        Console.WriteLine("  CHARACTER SANDBOX - GoudEngine");
        Console.WriteLine("========================================");
        Console.WriteLine("  W/S        Forward / Backward");
        Console.WriteLine("  A/D        Strafe left / right");
        Console.WriteLine("  Shift      Hold to run");
        Console.WriteLine("  Arrows     Camera yaw & pitch");
        Console.WriteLine("  G          Toggle grid");
        Console.WriteLine("  F          Toggle fog");
        Console.WriteLine("  ESC        Quit");
        Console.WriteLine("========================================");
        Console.WriteLine();

        // --- Window ---
        using var game = new GoudGame(1280, 720, "Character Sandbox");

        // --- Scene setup ---
        uint sceneId = game.CreateScene("main");
        game.SetCurrentScene(sceneId);

        // --- Skybox & fog ---
        game.ConfigureSkybox(enabled: true, r: 0.1f, g: 0.1f, b: 0.15f, a: 1.0f);
        game.ConfigureFog(enabled: true, r: 0.1f, g: 0.1f, b: 0.15f, density: 0.015f);

        // --- Grid (optional debug aid) ---
        game.ConfigureGrid(enabled: true, size: 40.0f, divisions: 40);

        // --- Lights ---
        // Point light above the scene (sun-like)
        uint sunLight = game.AddLight(
            0,                          // type: point
            0f, 15f, 0f,               // position: directly above center
            0f, -1f, 0f,               // direction
            1.0f, 0.95f, 0.85f,        // warm white color
            2.0f, 50f, 0f              // intensity, range, spotAngle
        );
        game.AddLightToScene(sceneId, sunLight);

        // Fill light on the opposite side
        uint fillLight = game.AddLight(
            0,
            -8f, 8f, 8f,
            0f, -1f, 0f,
            0.3f, 0.4f, 0.6f,          // cool blue fill
            1.0f, 30f, 0f
        );
        game.AddLightToScene(sceneId, fillLight);

        // --- Ground plane ---
        uint groundPlane = game.CreatePlane(0, 60f, 60f);
        game.SetObjectPosition(groundPlane, 0f, 0f, 0f);

        // Give the ground a greenish material
        uint groundMat = game.CreateMaterial(
            0,                          // type: standard/phong
            0.35f, 0.55f, 0.25f, 1f,   // earthy green
            16f,                        // shininess
            0f, 0.8f, 0.1f             // metallic, roughness, ao
        );
        game.SetObjectMaterial(groundPlane, groundMat);
        game.AddObjectToScene(sceneId, groundPlane);

        // --- Load model ---
        string modelPath = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory,
            "assets", "Character.glb"
        );

        if (!File.Exists(modelPath))
        {
            Console.WriteLine($"ERROR: Model not found at {modelPath}");
            Console.WriteLine("Place a .glb model at examples/csharp/character_sandbox/assets/Character.glb");
            return;
        }

        uint baseModel = game.LoadModel(modelPath);
        if (baseModel == 0)
        {
            Console.WriteLine("ERROR: LoadModel returned 0 - model loading failed.");
            return;
        }
        Console.WriteLine($"Loaded model (id={baseModel})");

        // --- List animations ---
        int animCount = game.GetAnimationCount(baseModel);
        Console.WriteLine($"Animation count: {animCount}");

        // Build a name -> index lookup so we can pick idle/walk/run by name.
        // Fallback: index 0 = idle, 1 = walk, 2 = run (common ordering).
        int idleAnim = 0;
        int walkAnim = Math.Min(1, animCount - 1);
        int runAnim  = Math.Min(2, animCount - 1);

        for (int i = 0; i < animCount; i++)
        {
            string name = game.GetAnimationName(baseModel, i);
            Console.WriteLine($"  [{i}] {name}");

            string lower = name.ToLowerInvariant();
            // Match most specific first
            if (lower == "idle_loop")
                idleAnim = i;
            else if (lower == "walk_loop")
                walkAnim = i;
            else if (lower == "sprint_loop" || lower == "jog_fwd_loop")
                runAnim = i;
        }

        Console.WriteLine($"Animation mapping -> idle={idleAnim}, walk={walkAnim}, run={runAnim}");

        // --- Player character ---
        // Use the base model as the player instance.
        game.SetModelPosition(baseModel, playerX, playerY, playerZ);
        game.SetModelScale(baseModel, 0.01f, 0.01f, 0.01f); // Quaternius model is in cm
        game.AddModelToScene(sceneId, baseModel);

        // Start with idle animation
        game.PlayAnimation(baseModel, idleAnim, true);

        // --- NPC characters ---
        // Place 3 NPCs at different positions, each playing idle.
        uint[] npcs = new uint[3];
        float[,] npcPositions = new float[,]
        {
            {  8f, 0f,  5f },
            { -6f, 0f,  9f },
            {  3f, 0f, -7f },
        };
        float[] npcFacings = { 45f, -90f, 180f };

        for (int i = 0; i < npcs.Length; i++)
        {
            npcs[i] = game.InstantiateModel(baseModel);
            if (npcs[i] == 0)
            {
                Console.WriteLine($"WARNING: Failed to instantiate NPC {i}");
                continue;
            }

            game.SetModelPosition(npcs[i], npcPositions[i, 0], npcPositions[i, 1], npcPositions[i, 2]);
            game.SetModelRotation(npcs[i], 0f, npcFacings[i], 0f);
            game.SetModelScale(npcs[i], 0.01f, 0.01f, 0.01f);
            game.AddModelToScene(sceneId, npcs[i]);

            // Each NPC loops idle
            game.PlayAnimation(npcs[i], idleAnim, true);
            // Slight speed variation so they are not perfectly in sync
            game.SetAnimationSpeed(npcs[i], 0.85f + 0.1f * i);

            Console.WriteLine($"NPC {i} instantiated (id={npcs[i]}) at ({npcPositions[i, 0]}, {npcPositions[i, 1]}, {npcPositions[i, 2]})");
        }

        // --- Decorative cubes (landmarks) ---
        uint pillarMat = game.CreateMaterial(0, 0.6f, 0.6f, 0.6f, 1f, 32f, 0f, 0.5f, 0.2f);
        float[] pillarX = { 15f, -15f, 15f, -15f };
        float[] pillarZ = { 15f, 15f, -15f, -15f };
        for (int i = 0; i < pillarX.Length; i++)
        {
            uint pillar = game.CreateCube(0, 1f, 4f, 1f);
            game.SetObjectPosition(pillar, pillarX[i], 2f, pillarZ[i]);
            game.SetObjectMaterial(pillar, pillarMat);
            game.AddObjectToScene(sceneId, pillar);
        }

        // --- State ---
        bool gridEnabled = true;
        bool fogEnabled  = true;

        int frameCount = 0;
        float fpsTimer = 0f;
        float lastFps  = 0f;

        // --- Game loop ---
        while (!game.ShouldClose())
        {
            float dt = game.DeltaTime;

            game.BeginFrame(0.1f, 0.1f, 0.15f, 1.0f);

            // Exit
            if (game.IsKeyPressed(Keys.Escape))
            {
                game.Close();
                continue;
            }

            // --- Toggle grid / fog ---
            if (game.IsKeyJustPressed(Keys.G))
            {
                gridEnabled = !gridEnabled;
                game.SetGridEnabled(gridEnabled);
                Console.WriteLine($"Grid {(gridEnabled ? "enabled" : "disabled")}");
            }
            if (game.IsKeyJustPressed(Keys.F))
            {
                fogEnabled = !fogEnabled;
                game.SetFogEnabled(fogEnabled);
                Console.WriteLine($"Fog {(fogEnabled ? "enabled" : "disabled")}");
            }

            // --- Camera rotation (arrow keys) ---
            if (game.IsKeyPressed(Keys.Left))
                cameraYaw += cameraRotSpeed * dt;
            if (game.IsKeyPressed(Keys.Right))
                cameraYaw -= cameraRotSpeed * dt;
            if (game.IsKeyPressed(Keys.Up))
                cameraPitch = Math.Clamp(cameraPitch + cameraRotSpeed * dt, 5f, 80f);
            if (game.IsKeyPressed(Keys.Down))
                cameraPitch = Math.Clamp(cameraPitch - cameraRotSpeed * dt, 5f, 80f);

            // --- Movement input ---
            float moveX = 0f;
            float moveZ = 0f;
            bool moving = false;
            bool running = game.IsKeyPressed(Keys.LeftShift) || game.IsKeyPressed(Keys.RightShift);

            // Camera-relative directions on the XZ plane
            float yawRad = cameraYaw * MathF.PI / 180f;
            float fwdX = MathF.Sin(yawRad);
            float fwdZ = MathF.Cos(yawRad);
            float rgtX =  MathF.Cos(yawRad);
            float rgtZ = -MathF.Sin(yawRad);

            if (game.IsKeyPressed(Keys.W)) { moveX += fwdX; moveZ += fwdZ; moving = true; }
            if (game.IsKeyPressed(Keys.S)) { moveX -= fwdX; moveZ -= fwdZ; moving = true; }
            if (game.IsKeyPressed(Keys.A)) { moveX += rgtX; moveZ += rgtZ; moving = true; }
            if (game.IsKeyPressed(Keys.D)) { moveX -= rgtX; moveZ -= rgtZ; moving = true; }

            // Normalize diagonal movement
            float moveLen = MathF.Sqrt(moveX * moveX + moveZ * moveZ);
            if (moveLen > 0.001f)
            {
                moveX /= moveLen;
                moveZ /= moveLen;
            }

            float speed = running ? runSpeed : walkSpeed;
            playerX += moveX * speed * dt;
            playerZ += moveZ * speed * dt;

            // Face movement direction
            if (moving)
            {
                float targetFacing = MathF.Atan2(moveX, moveZ) * 180f / MathF.PI;
                // Smooth rotation towards target facing
                float diff = targetFacing - playerFacing;
                // Wrap to [-180, 180]
                while (diff > 180f)  diff -= 360f;
                while (diff < -180f) diff += 360f;
                float rotSpeed = 720f; // degrees per second
                if (MathF.Abs(diff) < rotSpeed * dt)
                    playerFacing = targetFacing;
                else
                    playerFacing += MathF.Sign(diff) * rotSpeed * dt;
            }

            // --- Update player model transform ---
            game.SetModelPosition(baseModel, playerX, playerY, playerZ);
            game.SetModelRotation(baseModel, 0f, playerFacing, 0f);

            // --- Animation state machine ---
            AnimState desired;
            if (!moving)
                desired = AnimState.Idle;
            else if (running)
                desired = AnimState.Run;
            else
                desired = AnimState.Walk;

            if (desired != currentAnim)
            {
                int targetIndex = desired switch
                {
                    AnimState.Idle => idleAnim,
                    AnimState.Walk => walkAnim,
                    AnimState.Run  => runAnim,
                    _ => idleAnim
                };
                game.TransitionAnimation(baseModel, targetIndex, animTransitionTime);
                currentAnim = desired;
            }

            // --- Advance all animations ---
            game.UpdateAnimations(dt);

            // --- Third-person camera ---
            float pitchRad = cameraPitch * MathF.PI / 180f;
            float camOffX = -MathF.Sin(yawRad) * MathF.Cos(pitchRad) * cameraDistance;
            float camOffY = MathF.Sin(pitchRad) * cameraDistance + cameraHeight;
            float camOffZ = -MathF.Cos(yawRad) * MathF.Cos(pitchRad) * cameraDistance;

            float camX = playerX + camOffX;
            float camY = playerY + camOffY;
            float camZ = playerZ + camOffZ;

            game.SetCameraPosition3D(camX, camY, camZ);

            // Camera looks at player: compute pitch/yaw from camera to player
            float lookX = playerX - camX;
            float lookY = (playerY + 1.5f) - camY; // look slightly above feet
            float lookZ = playerZ - camZ;
            float lookDist = MathF.Sqrt(lookX * lookX + lookZ * lookZ);
            float lookPitch = MathF.Atan2(lookY, lookDist) * 180f / MathF.PI;
            float lookYaw = MathF.Atan2(lookX, lookZ) * 180f / MathF.PI;

            game.SetCameraRotation3D(lookPitch, lookYaw, 0f);

            // --- Render ---
            game.Render3D();
            game.EndFrame();

            // --- FPS counter ---
            frameCount++;
            fpsTimer += dt;
            if (fpsTimer >= 1.0f)
            {
                lastFps = frameCount / fpsTimer;
                Console.Write($"\rFPS: {lastFps:F1}  Player: ({playerX:F1}, {playerZ:F1})  Anim: {currentAnim}   ");
                frameCount = 0;
                fpsTimer = 0f;
            }
        }

        // --- Cleanup ---
        for (int i = 0; i < npcs.Length; i++)
        {
            if (npcs[i] != 0) game.DestroyModel(npcs[i]);
        }
        game.DestroyModel(baseModel);

        Console.WriteLine();
        Console.WriteLine("Character Sandbox ended.");
    }
}
