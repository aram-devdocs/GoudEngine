// Character Sandbox - Demonstrates model loading, skeletal animation,
// scene management, character movement, and third-person camera.
//
// This example loads a glTF (.glb) animated model, instantiates a
// configurable number of wandering NPCs, and lets the player walk
// around a lit ground plane with smooth animation transitions.
// It doubles as a scalable performance benchmark.
//
// CONTROLS:
//   W / S        - Move forward / backward
//   A / D        - Strafe left / right
//   Left Shift   - Hold to run
//   Arrow keys   - Rotate camera yaw (left/right) and pitch (up/down)
//   G            - Toggle debug grid
//   F            - Toggle fog
//   = / +        - Spawn 10 more NPCs
//   -            - Remove 10 NPCs
//   0-9          - Play animation  0- 9
//   Shift+0-9    - Play animation 10-19
//   Ctrl+0-9     - Play animation 20-29
//   Alt+0-9      - Play animation 30-39
//   Tab+0-9      - Play animation 40-44
//   ESC          - Quit
//
// CLI:
//   dotnet run -- --npcs 200

using System;
using System.Collections.Generic;
using System.IO;
using GoudEngine;

// ----------------------------------------------------------------
// NPC state
// ----------------------------------------------------------------
struct NpcState
{
    public uint modelId;
    public float x, z;
    public float facing;
    public float targetX, targetZ;
    public float idleTimer;
    public bool isMoving;
}

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
    // NPC placement bounds (ground plane is 200x200, keep NPCs inside)
    // ----------------------------------------------------------------
    static float npcBoundsMin = -95f;
    static float npcBoundsMax =  95f;

    // ----------------------------------------------------------------
    // Helpers
    // ----------------------------------------------------------------
    static int ParseNpcCount(string[] args, int defaultCount)
    {
        for (int i = 0; i < args.Length - 1; i++)
        {
            if (args[i] == "--npcs" && int.TryParse(args[i + 1], out int n) && n >= 0)
                return n;
        }
        return defaultCount;
    }

    static string ParseBackend(string[] args, string defaultBackend)
    {
        for (int i = 0; i < args.Length - 1; i++)
        {
            if (args[i] == "--backend")
                return args[i + 1].ToLowerInvariant();
        }
        return defaultBackend;
    }

    static NpcState CreateNpc(
        GoudGame game, uint baseModel, uint sceneId,
        Random rng, int animCount, int idleAnim, int walkAnim)
    {
        NpcState npc = new NpcState();

        npc.modelId = game.InstantiateModel(baseModel);
        if (npc.modelId == 0)
            return npc;

        npc.x = (float)(rng.NextDouble() * (npcBoundsMax - npcBoundsMin) + npcBoundsMin);
        npc.z = (float)(rng.NextDouble() * (npcBoundsMax - npcBoundsMin) + npcBoundsMin);
        npc.facing = (float)(rng.NextDouble() * 360.0);

        npc.targetX = (float)(rng.NextDouble() * (npcBoundsMax - npcBoundsMin) + npcBoundsMin);
        npc.targetZ = (float)(rng.NextDouble() * (npcBoundsMax - npcBoundsMin) + npcBoundsMin);

        // Start some NPCs idle, some moving
        npc.isMoving = rng.NextDouble() > 0.5;
        npc.idleTimer = npc.isMoving ? 0f : (float)(rng.NextDouble() * 2.0 + 1.0);

        game.SetModelPosition(npc.modelId, npc.x, 0f, npc.z);
        game.SetModelRotation(npc.modelId, 0f, npc.facing, 0f);
        game.SetModelScale(npc.modelId, 1f, 1f, 1f);
        game.AddModelToScene(sceneId, npc.modelId);

        // Random starting animation from the available set
        int startAnim = rng.Next(animCount);
        game.PlayAnimation(npc.modelId, startAnim, true);

        // Random speed multiplier 0.7 - 1.3
        float speedMul = 0.7f + (float)(rng.NextDouble() * 0.6);
        game.SetAnimationSpeed(npc.modelId, speedMul);

        // If moving, switch to walk animation
        if (npc.isMoving)
            game.TransitionAnimation(npc.modelId, walkAnim, 0.3f);

        return npc;
    }

    // ----------------------------------------------------------------
    // Entry point
    // ----------------------------------------------------------------
    static void Main(string[] args)
    {
        int initialNpcCount = ParseNpcCount(args, 50);
        string backendName = ParseBackend(args, "wgpu");

        Console.WriteLine("========================================");
        Console.WriteLine("  CHARACTER SANDBOX - GoudEngine");
        Console.WriteLine("========================================");
        Console.WriteLine("  W/S        Forward / Backward");
        Console.WriteLine("  A/D        Strafe left / right");
        Console.WriteLine("  Shift      Hold to run");
        Console.WriteLine("  Arrows     Camera yaw & pitch");
        Console.WriteLine("  G          Toggle grid");
        Console.WriteLine("  F          Toggle fog");
        Console.WriteLine("  =/+        Spawn 10 more NPCs");
        Console.WriteLine("  -          Remove 10 NPCs");
        Console.WriteLine("  1          Toggle frustum culling");
        Console.WriteLine("  2          Toggle GPU/CPU skinning");
        Console.WriteLine("  3          Toggle material sorting");
        Console.WriteLine("  4          Toggle animation LOD");
        Console.WriteLine("  5          Toggle shared anim eval");
        Console.WriteLine("  Shift+0-9  Play animation 10-19");
        Console.WriteLine("  Ctrl+0-9   Play animation 20-29");
        Console.WriteLine("  Alt+0-9    Play animation 30-39");
        Console.WriteLine("  Tab+0-9    Play animation 40-44");
        Console.WriteLine("  ESC        Quit");
        Console.WriteLine("========================================");
        Console.WriteLine($"  Initial NPCs: {initialNpcCount}");
        Console.WriteLine($"  Backend: {backendName}");
        Console.WriteLine("  Usage: dotnet run -- --npcs 200 --backend opengl");
        Console.WriteLine();

        // --- Window (backend selectable via --backend wgpu|opengl) ---
        var config = new EngineConfig()
            .SetSize(1280, 720)
            .SetTitle("Character Sandbox");
        if (backendName == "opengl")
        {
            config.SetRenderBackend(RenderBackendKind.OpenGlLegacy);
            config.SetWindowBackend(WindowBackendKind.GlfwLegacy);
            Console.WriteLine("Using OpenGL 3.3 + GLFW backend");
        }
        else
        {
            config.SetRenderBackend(RenderBackendKind.Wgpu);
            config.SetWindowBackend(WindowBackendKind.Winit);
            Console.WriteLine("Using wgpu (Vulkan/Metal/DX12) + winit backend");
        }
        using var game = config.Build();

        // --- Scene setup ---
        uint sceneId = game.CreateScene("main");
        game.SetCurrentScene(sceneId);

        // --- Skybox & fog ---
        game.ConfigureSkybox(enabled: true, r: 0.1f, g: 0.1f, b: 0.15f, a: 1.0f);
        game.ConfigureFog(enabled: true, r: 0.1f, g: 0.1f, b: 0.15f, density: 0.015f);

        // --- Grid (optional debug aid) ---
        game.ConfigureGrid(enabled: true, size: 200.0f, divisions: 100);

        // --- Lights ---
        // Point light above the scene (sun-like)
        uint sunLight = game.AddLight(
            0,                          // type: point
            0f, 25f, 0f,               // position: directly above center
            0f, -1f, 0f,               // direction
            1.0f, 0.95f, 0.85f,        // warm white color
            2.0f, 120f, 0f             // intensity, range, spotAngle
        );
        game.AddLightToScene(sceneId, sunLight);

        // Fill light on the opposite side
        uint fillLight = game.AddLight(
            0,
            -20f, 15f, 20f,
            0f, -1f, 0f,
            0.3f, 0.4f, 0.6f,          // cool blue fill
            1.0f, 80f, 0f
        );
        game.AddLightToScene(sceneId, fillLight);

        // --- Ground plane (200x200) ---
        uint groundPlane = game.CreatePlane(0, 200f, 200f);
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
        game.SetModelScale(baseModel, 1f, 1f, 1f); // Model is in meters (1.83m tall)
        game.AddModelToScene(sceneId, baseModel);

        // Start with idle animation
        game.PlayAnimation(baseModel, idleAnim, true);

        // --- NPC characters ---
        Random rng = new Random(42); // fixed seed for reproducibility
        List<NpcState> npcs = new List<NpcState>();

        for (int i = 0; i < initialNpcCount; i++)
        {
            NpcState npc = CreateNpc(game, baseModel, sceneId, rng, animCount, idleAnim, walkAnim);
            if (npc.modelId != 0)
                npcs.Add(npc);
            else
                Console.WriteLine($"WARNING: Failed to instantiate NPC {i}");
        }
        Console.WriteLine($"Spawned {npcs.Count} NPCs");

        // --- Decorative cubes (landmarks) ---
        uint pillarMat = game.CreateMaterial(0, 0.6f, 0.6f, 0.6f, 1f, 32f, 0f, 0.5f, 0.2f);
        float[] pillarX = { 40f, -40f, 40f, -40f };
        float[] pillarZ = { 40f, 40f, -40f, -40f };
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
        bool frustumCullingEnabled = true;
        bool gpuSkinning = true;       // 0=CPU, 1=GPU; starts GPU
        bool materialSortingEnabled = true;
        bool animationLodEnabled = true;
        bool sharedAnimEvalEnabled = true;

        int frameCount = 0;
        float fpsTimer = 0f;
        float lastFps  = 0f;

        float npcWalkSpeed = 3f; // NPCs walk slightly slower than the player
        float npcRotSpeed  = 360f; // degrees per second for NPC turning

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

            // --- Dynamic NPC spawning / removal ---
            if (game.IsKeyJustPressed(Keys.Equal))
            {
                int before = npcs.Count;
                for (int i = 0; i < 10; i++)
                {
                    NpcState npc = CreateNpc(game, baseModel, sceneId, rng, animCount, idleAnim, walkAnim);
                    if (npc.modelId != 0)
                        npcs.Add(npc);
                }
                Console.WriteLine($"Spawned NPCs: {before} -> {npcs.Count}");
            }
            if (game.IsKeyJustPressed(Keys.Minus))
            {
                int toRemove = Math.Min(10, npcs.Count);
                if (toRemove > 0)
                {
                    int before = npcs.Count;
                    for (int i = 0; i < toRemove; i++)
                    {
                        int last = npcs.Count - 1;
                        game.DestroyModel(npcs[last].modelId);
                        npcs.RemoveAt(last);
                    }
                    Console.WriteLine($"Removed NPCs: {before} -> {npcs.Count}");
                }
            }

            // --- Profiling toggles (digit keys 1-5 without modifier) ---
            {
                bool shift = game.IsKeyPressed(Keys.LeftShift) || game.IsKeyPressed(Keys.RightShift);
                bool ctrl  = game.IsKeyPressed(Keys.LeftControl) || game.IsKeyPressed(Keys.RightControl);
                bool alt   = game.IsKeyPressed(Keys.LeftAlt) || game.IsKeyPressed(Keys.RightAlt);
                bool tab   = game.IsKeyPressed(Keys.Tab);
                bool noMod = !shift && !ctrl && !alt && !tab;

                if (noMod && game.IsKeyJustPressed(Keys.Digit1))
                {
                    frustumCullingEnabled = !frustumCullingEnabled;
                    game.SetFrustumCullingEnabled(frustumCullingEnabled);
                    Console.WriteLine($"Frustum culling {(frustumCullingEnabled ? "ON" : "OFF")}");
                }
                if (noMod && game.IsKeyJustPressed(Keys.Digit2))
                {
                    gpuSkinning = !gpuSkinning;
                    game.SetSkinningMode(gpuSkinning ? 1u : 0u);
                    Console.WriteLine($"Skinning mode: {(gpuSkinning ? "GPU" : "CPU")}");
                }
                if (noMod && game.IsKeyJustPressed(Keys.Digit3))
                {
                    materialSortingEnabled = !materialSortingEnabled;
                    game.SetMaterialSortingEnabled(materialSortingEnabled);
                    Console.WriteLine($"Material sorting {(materialSortingEnabled ? "ON" : "OFF")}");
                }
                if (noMod && game.IsKeyJustPressed(Keys.Digit4))
                {
                    animationLodEnabled = !animationLodEnabled;
                    game.SetAnimationLodEnabled(animationLodEnabled);
                    Console.WriteLine($"Animation LOD {(animationLodEnabled ? "ON" : "OFF")}");
                }
                if (noMod && game.IsKeyJustPressed(Keys.Digit5))
                {
                    sharedAnimEvalEnabled = !sharedAnimEvalEnabled;
                    game.SetSharedAnimationEval(sharedAnimEvalEnabled);
                    Console.WriteLine($"Shared anim eval {(sharedAnimEvalEnabled ? "ON" : "OFF")}");
                }

                // --- Direct animation playback (modifier + digit keys) ---
                int animOffset = -1;
                if (tab)       animOffset = 40;
                else if (alt)  animOffset = 30;
                else if (ctrl) animOffset = 20;
                else if (shift) animOffset = 10;

                if (animOffset >= 0)
                {
                    Keys[] digitKeys = {
                        Keys.Digit0, Keys.Digit1, Keys.Digit2, Keys.Digit3, Keys.Digit4,
                        Keys.Digit5, Keys.Digit6, Keys.Digit7, Keys.Digit8, Keys.Digit9,
                    };
                    for (int d = 0; d < digitKeys.Length; d++)
                    {
                        if (game.IsKeyJustPressed(digitKeys[d]))
                        {
                            int idx = animOffset + d;
                            if (idx < animCount)
                            {
                                string name = game.GetAnimationName(baseModel, idx);
                                Console.WriteLine($"Playing animation [{idx}] {name}");
                                game.TransitionAnimation(baseModel, idx, animTransitionTime);
                            }
                            else
                            {
                                Console.WriteLine($"Animation index {idx} out of range (max {animCount - 1})");
                            }
                            break;
                        }
                    }
                }
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

            // --- Update NPC wander AI ---
            for (int i = 0; i < npcs.Count; i++)
            {
                NpcState npc = npcs[i];

                if (npc.isMoving)
                {
                    // Move toward target
                    float dx = npc.targetX - npc.x;
                    float dz = npc.targetZ - npc.z;
                    float dist = MathF.Sqrt(dx * dx + dz * dz);

                    if (dist < 1.0f)
                    {
                        // Arrived at target - idle for 1-3 seconds
                        npc.isMoving = false;
                        npc.idleTimer = 1.0f + (float)(rng.NextDouble() * 2.0);
                        game.TransitionAnimation(npc.modelId, idleAnim, 0.3f);
                    }
                    else
                    {
                        // Normalize direction and move
                        float ndx = dx / dist;
                        float ndz = dz / dist;

                        npc.x += ndx * npcWalkSpeed * dt;
                        npc.z += ndz * npcWalkSpeed * dt;

                        // Face movement direction (smooth rotation)
                        float targetFacing = MathF.Atan2(ndx, ndz) * 180f / MathF.PI;
                        float faceDiff = targetFacing - npc.facing;
                        while (faceDiff > 180f)  faceDiff -= 360f;
                        while (faceDiff < -180f) faceDiff += 360f;
                        if (MathF.Abs(faceDiff) < npcRotSpeed * dt)
                            npc.facing = targetFacing;
                        else
                            npc.facing += MathF.Sign(faceDiff) * npcRotSpeed * dt;

                        game.SetModelPosition(npc.modelId, npc.x, 0f, npc.z);
                        game.SetModelRotation(npc.modelId, 0f, npc.facing, 0f);
                    }
                }
                else
                {
                    // Idling - count down timer
                    npc.idleTimer -= dt;
                    if (npc.idleTimer <= 0f)
                    {
                        // Pick new random target and start walking
                        npc.targetX = (float)(rng.NextDouble() * (npcBoundsMax - npcBoundsMin) + npcBoundsMin);
                        npc.targetZ = (float)(rng.NextDouble() * (npcBoundsMax - npcBoundsMin) + npcBoundsMin);
                        npc.isMoving = true;
                        game.TransitionAnimation(npc.modelId, walkAnim, 0.3f);
                    }
                }

                npcs[i] = npc; // write back (struct copy)
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

            // --- FPS counter & stats overlay ---
            frameCount++;
            fpsTimer += dt;
            if (fpsTimer >= 1.0f)
            {
                lastFps = frameCount / fpsTimer;
                int draws = game.GetDrawCalls();
                int visible = game.GetVisibleObjectCount();
                int culled = game.GetCulledObjectCount();
                int total = visible + culled;
                Console.Write($"\rFPS: {lastFps:F1}  NPCs: {npcs.Count}  Draws: {draws}  Visible: {visible}/{total}   ");
                frameCount = 0;
                fpsTimer = 0f;
            }
        }

        // --- Cleanup ---
        for (int i = 0; i < npcs.Count; i++)
        {
            if (npcs[i].modelId != 0) game.DestroyModel(npcs[i].modelId);
        }
        game.DestroyModel(baseModel);

        Console.WriteLine();
        Console.WriteLine("Character Sandbox ended.");
    }
}
