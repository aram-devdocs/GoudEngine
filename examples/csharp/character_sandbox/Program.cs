using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text.Json;
using GoudEngine;

struct CrowdAgent
{
    public uint ModelId;
    public float X;
    public float Y;
    public float Z;
    public float Facing;
    public float TargetX;
    public float TargetZ;
    public float IdleTimer;
    public float MoveSpeed;
    public float SpeedMultiplier;
    public int IdleAnim;
    public int WalkAnim;
    public int RunAnim;
    public bool IsMoving;
    public bool IsAnimal;
    public float RotationX;
    public float RotationZ;
    public float YawOffset;
}

enum AssetCategory
{
    Humanoid,
    Animal,
    Building,
    Prop,
    Decoration,
}

sealed class AssetProfile
{
    public string Name { get; init; } = "";
    public string Path { get; init; } = "";
    public AssetCategory Category { get; init; }
    public bool IsDynamic { get; init; }
    public float Scale { get; init; } = 1f;
    public float? TargetHeight { get; init; }
    public float RotationX { get; init; }
    public float RotationY { get; init; }
    public float RotationZ { get; init; }
    public float GroundOffsetBias { get; init; }
    public string[] IdleClipNames { get; init; } = Array.Empty<string>();
    public string[] WalkClipNames { get; init; } = Array.Empty<string>();
    public string[] RunClipNames { get; init; } = Array.Empty<string>();
}

sealed class RigConfig
{
    public string Name { get; init; } = "";
    public string Path { get; init; } = "";
    public uint SourceModelId { get; init; }
    public int AnimCount { get; init; }
    public int IdleAnim { get; init; }
    public int WalkAnim { get; init; }
    public int RunAnim { get; init; }
    public float Scale { get; init; } = 1f;
    public float MoveSpeed { get; init; } = 3f;
    public bool IsAnimal { get; init; }
    public float GroundOffsetY { get; init; }
    public float RotationX { get; init; }
    public float RotationY { get; init; }
    public float RotationZ { get; init; }
}

sealed class AssetDiagnostics
{
    public string Name { get; init; } = "";
    public string Path { get; init; } = "";
    public string Category { get; init; } = "";
    public bool IsDynamic { get; init; }
    public float RawWidth { get; init; }
    public float RawHeight { get; init; }
    public float RawDepth { get; init; }
    public float FinalScale { get; init; }
    public float GroundOffsetY { get; init; }
    public int AnimationCount { get; init; }
    public int IdleAnimation { get; init; }
    public int WalkAnimation { get; init; }
    public int RunAnimation { get; init; }
    public string[] AnimationNames { get; init; } = Array.Empty<string>();
}

enum ScenarioKind
{
    Baseline,
    FbxStaticDiagnostic,
    FbxAnimalDiagnostic,
    MixedCorrectnessDiagnostic,
    FullGameStress,
    GameLikeVillage,
    VariedAnimationCrowd,
    ThroneCurrentMix,
    WorstCaseAllVisible,
    SpawnDespawnChurn,
}

enum SpawnLayout
{
    Wide,
    Compact,
    Village,
}

enum StaticScatterKind
{
    Building,
    Prop,
    Decoration,
}

sealed class SandboxOptions
{
    public int InitialNpcCount { get; set; } = 80;
    public string BackendName { get; set; } = "wgpu";
    public ScenarioKind Scenario { get; set; } = ScenarioKind.FullGameStress;
    public float DurationSeconds { get; set; } = 0f;
    public string? MetricsOutPath { get; set; }
    public string ThroneAssetsRoot { get; set; } = Program.DefaultThroneAssetsRoot;
    public int Seed { get; set; } = 42;
    public AssetCategory? OnlyCategory { get; set; }
    public string? OnlyAsset { get; set; }
    public bool NoHumanoids { get; set; }
    public bool NoAnimals { get; set; }
    public bool NoStatics { get; set; }

    public bool BenchmarkMode =>
        DurationSeconds > 0f ||
        !string.IsNullOrWhiteSpace(MetricsOutPath);
}

sealed class ScenarioState
{
    public required RigConfig PlayerRig { get; init; }
    public uint PlayerModelId { get; init; }
    public List<RigConfig> HumanoidRigs { get; } = new();
    public List<RigConfig> AnimalRigs { get; } = new();
    public List<uint> SourceModelIds { get; } = new();
    public List<CrowdAgent> Agents { get; } = new();
    public List<uint> StaticInstances { get; } = new();
    public List<string> LoadedAssets { get; } = new();
    public List<string> FailedAssets { get; } = new();
    public Dictionary<uint, AssetProfile> SourceProfiles { get; } = new();
    public List<AssetDiagnostics> AssetDiagnostics { get; } = new();
    public int StaticBuildingCount { get; set; }
    public int StaticPropCount { get; set; }
    public int StaticDecorationCount { get; set; }
}

sealed class MetricsAccumulator
{
    public List<double> FrameMs { get; } = new();
    public List<double> DrawCalls { get; } = new();
    public List<double> VisibleObjects { get; } = new();
    public List<double> CulledObjects { get; } = new();
    public List<double> InstancedDrawCalls { get; } = new();
    public List<double> ActiveInstances { get; } = new();
    public List<double> BoneMatrixUploads { get; } = new();
    public List<double> AnimationEvaluations { get; } = new();
    public List<double> AnimationEvaluationsSaved { get; } = new();
}

class Program
{
    enum AnimState
    {
        Idle,
        Walk,
        Run,
    }

    public const string DefaultThroneAssetsRoot = "/Users/aramhammoudeh/dev/game/throne_ge/assets/models";

    static float cameraPitch = 25f;
    static float cameraYaw = 0f;
    static float cameraDistance = 12f;
    static float cameraHeight = 6f;
    static float cameraRotSpeed = 90f;

    static float walkSpeed = 4f;
    static float runSpeed = 9f;

    static float playerX = 0f;
    static float playerY = 0f;
    static float playerZ = 0f;
    static float playerFacing = 0f;

    static AnimState currentAnim = AnimState.Idle;
    static float animTransitionTime = 0.25f;

    static float npcBoundsMin = -95f;
    static float npcBoundsMax = 95f;

    static void Main(string[] args)
    {
        SandboxOptions options = ParseOptions(args);
        PrintStartup(options);

        if (options.Scenario == ScenarioKind.FullGameStress ||
            options.Scenario == ScenarioKind.FbxStaticDiagnostic ||
            options.Scenario == ScenarioKind.FbxAnimalDiagnostic ||
            options.Scenario == ScenarioKind.MixedCorrectnessDiagnostic)
        {
            cameraDistance = 36f;
            cameraHeight = 13f;
            cameraPitch = 20f;
        }

        var config = new EngineConfig()
            .SetSize(1280, 720)
            .SetTitle("Character Sandbox");

        if (options.BackendName == "opengl")
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

        uint sceneId = game.CreateScene("main");
        game.SetCurrentScene(sceneId);

        game.ConfigureSkybox(enabled: true, r: 0.56f, g: 0.71f, b: 0.90f, a: 1.0f);
        game.ConfigureFog(enabled: false, r: 0.56f, g: 0.71f, b: 0.90f, density: 0.005f);
        game.ConfigureGrid(enabled: false, size: 200.0f, divisions: 100);

        uint sunLight = game.AddLight(
            0,
            0f, 25f, 0f,
            0f, -1f, 0f,
            1.0f, 0.97f, 0.90f,
            4.5f, 120f, 0f
        );
        game.AddLightToScene(sceneId, sunLight);

        uint fillLight = game.AddLight(
            0,
            -20f, 15f, 20f,
            0f, -1f, 0f,
            0.55f, 0.62f, 0.72f,
            2.0f, 120f, 0f
        );
        game.AddLightToScene(sceneId, fillLight);

        uint groundPlane = game.CreatePlane(0, 200f, 200f);
        game.SetObjectPosition(groundPlane, 0f, 0f, 0f);

        uint groundMat = game.CreateMaterial(
            0,
            0.43f, 0.62f, 0.34f, 1f,
            16f,
            0f, 0.8f, 0.1f
        );
        game.SetObjectMaterial(groundPlane, groundMat);
        game.AddObjectToScene(sceneId, groundPlane);

        game.SetFrustumCullingEnabled(true);
        game.SetSkinningMode(1u);
        game.SetMaterialSortingEnabled(true);
        game.SetAnimationLodEnabled(true);
        game.SetSharedAnimationEval(true);

        Random rng = new(options.Seed);
        ScenarioState state = BuildScenario(game, options, sceneId, rng);
        Console.WriteLine(
            $"Scenario loaded: humanoids={state.Agents.Count(a => !a.IsAnimal)} " +
            $"animals={state.Agents.Count(a => a.IsAnimal)} " +
            $"staticBuildings={state.StaticBuildingCount} staticProps={state.StaticPropCount} " +
            $"staticDecorations={state.StaticDecorationCount}"
        );

        bool gridEnabled = false;
        bool fogEnabled = false;
        bool frustumCullingEnabled = true;
        bool gpuSkinning = true;
        bool materialSortingEnabled = true;
        bool animationLodEnabled = true;
        bool sharedAnimEvalEnabled = true;

        float npcRotSpeed = 360f;
        float fpsTimer = 0f;
        int fpsFrames = 0;
        double lastFps = 0.0;
        float churnTimer = 0f;
        bool churnGrowing = true;

        var benchmarkClock = Stopwatch.StartNew();
        MetricsAccumulator metrics = new();

        while (!game.ShouldClose())
        {
            long frameStart = Stopwatch.GetTimestamp();
            float dt = game.DeltaTime;

            game.BeginFrame(0.1f, 0.1f, 0.15f, 1.0f);

            if (game.IsKeyPressed(Keys.Escape))
            {
                game.Close();
                continue;
            }

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

            if (game.IsKeyJustPressed(Keys.Equal))
            {
                int before = state.Agents.Count;
                for (int i = 0; i < 10; i++)
                {
                    RigConfig rig = PickSpawnRig(state, before + i);
                    CrowdAgent agent = CreateCrowdAgent(
                        game,
                        rig,
                        sceneId,
                        rng,
                        LayoutForScenario(options.Scenario),
                        before + i
                    );
                    if (agent.ModelId != 0)
                    {
                        state.Agents.Add(agent);
                    }
                }
                Console.WriteLine($"Spawned agents: {before} -> {state.Agents.Count}");
            }

            if (game.IsKeyJustPressed(Keys.Minus))
            {
                int toRemove = Math.Min(10, state.Agents.Count);
                if (toRemove > 0)
                {
                    int before = state.Agents.Count;
                    for (int i = 0; i < toRemove; i++)
                    {
                        int last = state.Agents.Count - 1;
                        game.DestroyModel(state.Agents[last].ModelId);
                        state.Agents.RemoveAt(last);
                    }
                    Console.WriteLine($"Removed agents: {before} -> {state.Agents.Count}");
                }
            }

            bool shift = game.IsKeyPressed(Keys.LeftShift) || game.IsKeyPressed(Keys.RightShift);
            bool ctrl = game.IsKeyPressed(Keys.LeftControl) || game.IsKeyPressed(Keys.RightControl);
            bool alt = game.IsKeyPressed(Keys.LeftAlt) || game.IsKeyPressed(Keys.RightAlt);
            bool tab = game.IsKeyPressed(Keys.Tab);
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

            int animOffset = -1;
            if (tab)
            {
                animOffset = 40;
            }
            else if (alt)
            {
                animOffset = 30;
            }
            else if (ctrl)
            {
                animOffset = 20;
            }
            else if (shift)
            {
                animOffset = 10;
            }

            if (animOffset >= 0)
            {
                Keys[] digitKeys =
                {
                    Keys.Digit0, Keys.Digit1, Keys.Digit2, Keys.Digit3, Keys.Digit4,
                    Keys.Digit5, Keys.Digit6, Keys.Digit7, Keys.Digit8, Keys.Digit9,
                };
                for (int digit = 0; digit < digitKeys.Length; digit++)
                {
                    if (!game.IsKeyJustPressed(digitKeys[digit]))
                    {
                        continue;
                    }

                    int targetAnim = animOffset + digit;
                    if (targetAnim < state.PlayerRig.AnimCount)
                    {
                        string name = game.GetAnimationName(state.PlayerModelId, targetAnim);
                        Console.WriteLine($"Playing animation [{targetAnim}] {name}");
                        game.TransitionAnimation(state.PlayerModelId, targetAnim, animTransitionTime);
                    }
                    else
                    {
                        Console.WriteLine(
                            $"Animation index {targetAnim} out of range (max {state.PlayerRig.AnimCount - 1})"
                        );
                    }
                    break;
                }
            }

            if (game.IsKeyPressed(Keys.Left))
            {
                cameraYaw += cameraRotSpeed * dt;
            }
            if (game.IsKeyPressed(Keys.Right))
            {
                cameraYaw -= cameraRotSpeed * dt;
            }
            if (game.IsKeyPressed(Keys.Up))
            {
                cameraPitch = Math.Clamp(cameraPitch + cameraRotSpeed * dt, 5f, 80f);
            }
            if (game.IsKeyPressed(Keys.Down))
            {
                cameraPitch = Math.Clamp(cameraPitch - cameraRotSpeed * dt, 5f, 80f);
            }

            float moveX = 0f;
            float moveZ = 0f;
            bool moving = false;
            bool running = game.IsKeyPressed(Keys.LeftShift) || game.IsKeyPressed(Keys.RightShift);

            float yawRad = cameraYaw * MathF.PI / 180f;
            float fwdX = MathF.Sin(yawRad);
            float fwdZ = MathF.Cos(yawRad);
            float rgtX = MathF.Cos(yawRad);
            float rgtZ = -MathF.Sin(yawRad);

            if (game.IsKeyPressed(Keys.W))
            {
                moveX += fwdX;
                moveZ += fwdZ;
                moving = true;
            }
            if (game.IsKeyPressed(Keys.S))
            {
                moveX -= fwdX;
                moveZ -= fwdZ;
                moving = true;
            }
            if (game.IsKeyPressed(Keys.A))
            {
                moveX += rgtX;
                moveZ += rgtZ;
                moving = true;
            }
            if (game.IsKeyPressed(Keys.D))
            {
                moveX -= rgtX;
                moveZ -= rgtZ;
                moving = true;
            }

            float moveLen = MathF.Sqrt(moveX * moveX + moveZ * moveZ);
            if (moveLen > 0.001f)
            {
                moveX /= moveLen;
                moveZ /= moveLen;
            }

            float speed = running ? runSpeed : walkSpeed;
            playerX += moveX * speed * dt;
            playerZ += moveZ * speed * dt;

            if (moving)
            {
                float targetFacing = MathF.Atan2(moveX, moveZ) * 180f / MathF.PI;
                float diff = WrapDegrees(targetFacing - playerFacing);
                float rotationSpeed = 720f;
                if (MathF.Abs(diff) < rotationSpeed * dt)
                {
                    playerFacing = targetFacing;
                }
                else
                {
                    playerFacing += MathF.Sign(diff) * rotationSpeed * dt;
                }
            }

            game.SetModelPosition(state.PlayerModelId, playerX, playerY, playerZ);
            game.SetModelRotation(
                state.PlayerModelId,
                state.PlayerRig.RotationX,
                playerFacing + state.PlayerRig.RotationY,
                state.PlayerRig.RotationZ
            );

            AnimState desired = !moving
                ? AnimState.Idle
                : (running ? AnimState.Run : AnimState.Walk);

            if (desired != currentAnim && state.PlayerRig.AnimCount > 0)
            {
                int targetIndex = desired switch
                {
                    AnimState.Idle => state.PlayerRig.IdleAnim,
                    AnimState.Walk => state.PlayerRig.WalkAnim,
                    AnimState.Run => state.PlayerRig.RunAnim,
                    _ => state.PlayerRig.IdleAnim,
                };
                game.TransitionAnimation(state.PlayerModelId, targetIndex, animTransitionTime);
                currentAnim = desired;
            }

            if (options.Scenario == ScenarioKind.SpawnDespawnChurn)
            {
                churnTimer += dt;
                if (churnTimer >= 1.5f)
                {
                    churnTimer = 0f;
                    int batchSize = 12;
                    if (churnGrowing || state.Agents.Count < batchSize)
                    {
                        for (int i = 0; i < batchSize; i++)
                        {
                            RigConfig rig = PickSpawnRig(state, state.Agents.Count + i);
                            CrowdAgent agent = CreateCrowdAgent(
                                game,
                                rig,
                                sceneId,
                                rng,
                                SpawnLayout.Village,
                                state.Agents.Count + i
                            );
                            if (agent.ModelId != 0)
                            {
                                state.Agents.Add(agent);
                            }
                        }
                    }
                    else
                    {
                        int toRemove = Math.Min(batchSize, state.Agents.Count);
                        for (int i = 0; i < toRemove; i++)
                        {
                            int last = state.Agents.Count - 1;
                            game.DestroyModel(state.Agents[last].ModelId);
                            state.Agents.RemoveAt(last);
                        }
                    }

                    churnGrowing = !churnGrowing;
                }
            }

            for (int i = 0; i < state.Agents.Count; i++)
            {
                CrowdAgent agent = state.Agents[i];

                if (agent.IsMoving)
                {
                    float dx = agent.TargetX - agent.X;
                    float dz = agent.TargetZ - agent.Z;
                    float distance = MathF.Sqrt(dx * dx + dz * dz);

                    if (distance < 1.0f)
                    {
                        agent.IsMoving = false;
                        agent.IdleTimer = 0.75f + (float)(rng.NextDouble() * 1.75);
                        if (agent.IdleAnim >= 0)
                        {
                            game.TransitionAnimation(agent.ModelId, agent.IdleAnim, 0.25f);
                        }
                    }
                    else
                    {
                        float ndx = dx / distance;
                        float ndz = dz / distance;
                        float motionSpeed = agent.MoveSpeed * agent.SpeedMultiplier;

                        agent.X += ndx * motionSpeed * dt;
                        agent.Z += ndz * motionSpeed * dt;

                        float targetFacing = MathF.Atan2(ndx, ndz) * 180f / MathF.PI;
                        float faceDiff = WrapDegrees(targetFacing - agent.Facing);
                        if (MathF.Abs(faceDiff) < npcRotSpeed * dt)
                        {
                            agent.Facing = targetFacing;
                        }
                        else
                        {
                            agent.Facing += MathF.Sign(faceDiff) * npcRotSpeed * dt;
                        }

                        game.SetModelPosition(agent.ModelId, agent.X, agent.Y, agent.Z);
                        game.SetModelRotation(
                            agent.ModelId,
                            agent.RotationX,
                            agent.Facing + agent.YawOffset,
                            agent.RotationZ
                        );
                    }
                }
                else
                {
                    agent.IdleTimer -= dt;
                    if (agent.IdleTimer <= 0f)
                    {
                        (agent.TargetX, agent.TargetZ) = PickTarget(
                            rng,
                            LayoutForScenario(options.Scenario),
                            agent.IsAnimal
                        );
                        agent.IsMoving = true;
                        if (agent.WalkAnim >= 0)
                        {
                            game.TransitionAnimation(agent.ModelId, agent.WalkAnim, 0.25f);
                        }
                    }
                }

                state.Agents[i] = agent;
            }

            game.UpdateAnimations(dt);

            float pitchRad = cameraPitch * MathF.PI / 180f;
            float camOffX = -MathF.Sin(yawRad) * MathF.Cos(pitchRad) * cameraDistance;
            float camOffY = MathF.Sin(pitchRad) * cameraDistance + cameraHeight;
            float camOffZ = -MathF.Cos(yawRad) * MathF.Cos(pitchRad) * cameraDistance;

            float camX = playerX + camOffX;
            float camY = playerY + camOffY;
            float camZ = playerZ + camOffZ;

            game.SetCameraPosition3D(camX, camY, camZ);

            float lookX = playerX - camX;
            float lookY = (playerY + 1.5f) - camY;
            float lookZ = playerZ - camZ;
            float lookDist = MathF.Sqrt(lookX * lookX + lookZ * lookZ);
            float lookPitch = MathF.Atan2(lookY, lookDist) * 180f / MathF.PI;
            float lookYaw = MathF.Atan2(lookX, lookZ) * 180f / MathF.PI;

            game.SetCameraRotation3D(lookPitch, lookYaw, 0f);
            game.Render3D();
            game.EndFrame();

            double frameMs = (Stopwatch.GetTimestamp() - frameStart) * 1000.0 / Stopwatch.Frequency;
            fpsTimer += (float)(frameMs / 1000.0);
            fpsFrames++;

            metrics.FrameMs.Add(frameMs);
            metrics.DrawCalls.Add(game.GetDrawCalls());
            metrics.VisibleObjects.Add(game.GetVisibleObjectCount());
            metrics.CulledObjects.Add(game.GetCulledObjectCount());
            metrics.InstancedDrawCalls.Add(game.GetInstancedDrawCalls());
            metrics.ActiveInstances.Add(game.GetActiveInstanceCount());
            metrics.BoneMatrixUploads.Add(game.GetBoneMatrixUploadCount());
            metrics.AnimationEvaluations.Add(game.GetAnimationEvaluationCount());
            metrics.AnimationEvaluationsSaved.Add(game.GetAnimationEvaluationSavedCount());

            if (fpsTimer >= 1.0f)
            {
                lastFps = fpsFrames / fpsTimer;
                int draws = game.GetDrawCalls();
                int visible = game.GetVisibleObjectCount();
                int culled = game.GetCulledObjectCount();
                int instanced = game.GetInstancedDrawCalls();
                int activeInstances = game.GetActiveInstanceCount();
                int animEval = game.GetAnimationEvaluationCount();
                int animSaved = game.GetAnimationEvaluationSavedCount();
                int boneUploads = game.GetBoneMatrixUploadCount();
                int total = visible + culled;

                string status =
                    $"FPS: {lastFps:F1}  Agents: {state.Agents.Count}  Draws: {draws}  " +
                    $"Visible: {visible}/{total}  Instanced: {instanced}  ActiveInstances: {activeInstances}  " +
                    $"AnimEval: {animEval}  Saved: {animSaved}  BoneUploads: {boneUploads}";

                if (options.BenchmarkMode)
                {
                    Console.WriteLine(status);
                }
                else
                {
                    Console.Write($"\r{status}   ");
                }

                fpsFrames = 0;
                fpsTimer = 0f;
            }

            if (options.BenchmarkMode && options.DurationSeconds > 0f)
            {
                if (benchmarkClock.Elapsed.TotalSeconds >= options.DurationSeconds)
                {
                    game.Close();
                }
            }
        }

        double elapsedSeconds = benchmarkClock.Elapsed.TotalSeconds;

        if (!string.IsNullOrWhiteSpace(options.MetricsOutPath))
        {
            WriteMetricsReport(options, state, metrics, elapsedSeconds, lastFps);
            Console.WriteLine($"Wrote metrics to {options.MetricsOutPath}");
        }

        foreach (CrowdAgent agent in state.Agents)
        {
            if (agent.ModelId != 0)
            {
                game.DestroyModel(agent.ModelId);
            }
        }

        foreach (uint modelId in state.StaticInstances)
        {
            if (modelId != 0)
            {
                game.DestroyModel(modelId);
            }
        }

        foreach (uint modelId in state.SourceModelIds.Distinct().Reverse())
        {
            if (modelId != 0)
            {
                game.DestroyModel(modelId);
            }
        }

        Console.WriteLine();
        Console.WriteLine("Character Sandbox ended.");
    }

    static SandboxOptions ParseOptions(string[] args)
    {
        SandboxOptions options = new();

        for (int i = 0; i < args.Length; i++)
        {
            string arg = args[i];
            if (arg == "--npcs" && i + 1 < args.Length && int.TryParse(args[i + 1], out int npcCount))
            {
                options.InitialNpcCount = Math.Max(0, npcCount);
                i++;
            }
            else if (arg == "--backend" && i + 1 < args.Length)
            {
                options.BackendName = args[i + 1].ToLowerInvariant();
                i++;
            }
            else if (arg == "--scenario" && i + 1 < args.Length)
            {
                options.Scenario = ParseScenario(args[i + 1]);
                i++;
            }
            else if (arg == "--duration" && i + 1 < args.Length && float.TryParse(args[i + 1], out float seconds))
            {
                options.DurationSeconds = Math.Max(0f, seconds);
                i++;
            }
            else if (arg == "--metrics-out" && i + 1 < args.Length)
            {
                options.MetricsOutPath = args[i + 1];
                i++;
            }
            else if (arg == "--throne-assets" && i + 1 < args.Length)
            {
                options.ThroneAssetsRoot = args[i + 1];
                i++;
            }
            else if (arg == "--seed" && i + 1 < args.Length && int.TryParse(args[i + 1], out int seed))
            {
                options.Seed = seed;
                i++;
            }
            else if (arg == "--only-category" && i + 1 < args.Length)
            {
                options.OnlyCategory = ParseCategory(args[i + 1]);
                i++;
            }
            else if (arg == "--only-asset" && i + 1 < args.Length)
            {
                options.OnlyAsset = args[i + 1];
                i++;
            }
            else if (arg == "--no-humanoids")
            {
                options.NoHumanoids = true;
            }
            else if (arg == "--no-animals")
            {
                options.NoAnimals = true;
            }
            else if (arg == "--no-statics")
            {
                options.NoStatics = true;
            }
        }

        if (options.BenchmarkMode && options.DurationSeconds <= 0f)
        {
            options.DurationSeconds = 20f;
        }

        return options;
    }

    static ScenarioKind ParseScenario(string raw)
    {
        return raw.ToLowerInvariant() switch
        {
            "baseline" => ScenarioKind.Baseline,
            "fbx-static-diagnostic" => ScenarioKind.FbxStaticDiagnostic,
            "static-diagnostic" => ScenarioKind.FbxStaticDiagnostic,
            "fbx-animal-diagnostic" => ScenarioKind.FbxAnimalDiagnostic,
            "animal-diagnostic" => ScenarioKind.FbxAnimalDiagnostic,
            "mixed-correctness-diagnostic" => ScenarioKind.MixedCorrectnessDiagnostic,
            "mixed-diagnostic" => ScenarioKind.MixedCorrectnessDiagnostic,
            "full-game-stress" => ScenarioKind.FullGameStress,
            "full-game" => ScenarioKind.FullGameStress,
            "stress" => ScenarioKind.FullGameStress,
            "game-like" => ScenarioKind.GameLikeVillage,
            "game-like-village" => ScenarioKind.GameLikeVillage,
            "village" => ScenarioKind.GameLikeVillage,
            "live" => ScenarioKind.GameLikeVillage,
            "varied-animation" => ScenarioKind.VariedAnimationCrowd,
            "varied-animation-crowd" => ScenarioKind.VariedAnimationCrowd,
            "throne-current-mix" => ScenarioKind.ThroneCurrentMix,
            "worst-case" => ScenarioKind.WorstCaseAllVisible,
            "worst-case-all-visible" => ScenarioKind.WorstCaseAllVisible,
            "spawn-despawn" => ScenarioKind.SpawnDespawnChurn,
            "spawn-despawn-churn" => ScenarioKind.SpawnDespawnChurn,
            _ => ScenarioKind.FullGameStress,
        };
    }

    static AssetCategory ParseCategory(string raw)
    {
        return raw.ToLowerInvariant() switch
        {
            "humanoid" => AssetCategory.Humanoid,
            "animal" => AssetCategory.Animal,
            "building" => AssetCategory.Building,
            "prop" => AssetCategory.Prop,
            "decoration" => AssetCategory.Decoration,
            _ => throw new ArgumentException($"Unknown category '{raw}'"),
        };
    }

    static void PrintStartup(SandboxOptions options)
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
        Console.WriteLine("  =/+        Spawn 10 more agents");
        Console.WriteLine("  -          Remove 10 agents");
        Console.WriteLine("  1          Toggle frustum culling");
        Console.WriteLine("  2          Toggle GPU/CPU skinning");
        Console.WriteLine("  3          Toggle material sorting");
        Console.WriteLine("  4          Toggle animation LOD");
        Console.WriteLine("  5          Toggle shared anim eval");
        Console.WriteLine("  ESC        Quit");
        Console.WriteLine("========================================");
        Console.WriteLine($"  Initial crowd: {options.InitialNpcCount}");
        Console.WriteLine($"  Backend: {options.BackendName}");
        Console.WriteLine($"  Scenario: {ScenarioLabel(options.Scenario)}");
        if (options.BenchmarkMode)
        {
            Console.WriteLine($"  Benchmark duration: {options.DurationSeconds:F1}s");
        }
        if (!string.IsNullOrWhiteSpace(options.MetricsOutPath))
        {
            Console.WriteLine($"  Metrics output: {options.MetricsOutPath}");
        }
        if (options.OnlyCategory.HasValue)
        {
            Console.WriteLine($"  Only category: {options.OnlyCategory.Value}");
        }
        if (!string.IsNullOrWhiteSpace(options.OnlyAsset))
        {
            Console.WriteLine($"  Only asset: {options.OnlyAsset}");
        }
        if (options.NoHumanoids || options.NoAnimals || options.NoStatics)
        {
            Console.WriteLine(
                $"  Filters: noHumanoids={options.NoHumanoids} noAnimals={options.NoAnimals} noStatics={options.NoStatics}"
            );
        }
        Console.WriteLine($"  Throne assets: {options.ThroneAssetsRoot}");
        Console.WriteLine(
            "  Usage: dotnet run -- --scenario full-game-stress --npcs 120"
        );
        Console.WriteLine();
    }

    static string ScenarioLabel(ScenarioKind scenario)
    {
        return scenario switch
        {
            ScenarioKind.Baseline => "baseline",
            ScenarioKind.FbxStaticDiagnostic => "fbx-static-diagnostic",
            ScenarioKind.FbxAnimalDiagnostic => "fbx-animal-diagnostic",
            ScenarioKind.MixedCorrectnessDiagnostic => "mixed-correctness-diagnostic",
            ScenarioKind.FullGameStress => "full-game-stress",
            ScenarioKind.GameLikeVillage => "game-like-village",
            ScenarioKind.VariedAnimationCrowd => "varied-animation",
            ScenarioKind.ThroneCurrentMix => "throne-current-mix",
            ScenarioKind.WorstCaseAllVisible => "worst-case-all-visible",
            ScenarioKind.SpawnDespawnChurn => "spawn-despawn-churn",
            _ => "full-game-stress",
        };
    }

    static SpawnLayout LayoutForScenario(ScenarioKind scenario)
    {
        return scenario switch
        {
            ScenarioKind.WorstCaseAllVisible => SpawnLayout.Compact,
            ScenarioKind.FbxStaticDiagnostic => SpawnLayout.Village,
            ScenarioKind.FbxAnimalDiagnostic => SpawnLayout.Village,
            ScenarioKind.MixedCorrectnessDiagnostic => SpawnLayout.Village,
            ScenarioKind.FullGameStress => SpawnLayout.Village,
            ScenarioKind.GameLikeVillage => SpawnLayout.Village,
            _ => SpawnLayout.Wide,
        };
    }

    static AssetProfile DynamicProfile(
        string path,
        AssetCategory category,
        float scale,
        string[] idleClipNames,
        string[] walkClipNames,
        string[] runClipNames
    )
    {
        AssetProfile profile = new AssetProfile
        {
            Name = Path.GetFileNameWithoutExtension(path),
            Path = path,
            Category = category,
            IsDynamic = true,
            Scale = scale,
            GroundOffsetBias = category == AssetCategory.Humanoid ? 0.02f : 0.05f,
            IdleClipNames = idleClipNames,
            WalkClipNames = walkClipNames,
            RunClipNames = runClipNames,
        };
        return ApplyAssetProfileOverrides(profile);
    }

    static AssetProfile StaticProfile(string path, AssetCategory category, float scale)
    {
        AssetProfile profile = new AssetProfile
        {
            Name = Path.GetFileNameWithoutExtension(path),
            Path = path,
            Category = category,
            IsDynamic = false,
            Scale = scale,
            GroundOffsetBias = 0.0f,
        };
        return ApplyAssetProfileOverrides(profile);
    }

    static AssetProfile ApplyAssetProfileOverrides(AssetProfile profile)
    {
        string name = Path.GetFileNameWithoutExtension(profile.Path).ToLowerInvariant();
        return new AssetProfile
        {
            Name = profile.Name,
            Path = profile.Path,
            Category = profile.Category,
            IsDynamic = profile.IsDynamic,
            Scale = ResolveAssetScale(profile.Category, name, profile.Scale),
            TargetHeight = ResolveTargetHeight(profile.Category, name, profile.TargetHeight),
            RotationX = profile.RotationX,
            RotationY = profile.RotationY,
            RotationZ = profile.RotationZ,
            GroundOffsetBias = ResolveGroundBias(profile.Category, name, profile.GroundOffsetBias),
            IdleClipNames = profile.IdleClipNames,
            WalkClipNames = profile.WalkClipNames,
            RunClipNames = profile.RunClipNames,
        };
    }

    static float ResolveAssetScale(AssetCategory category, string name, float currentScale)
    {
        if (category == AssetCategory.Building)
        {
            return name switch
            {
                "bell_tower" => 0.014f,
                "inn" or "blacksmith" or "mill" or "sawmill" or "stable" => 0.0125f,
                _ => 0.012f,
            };
        }

        if (category == AssetCategory.Prop || category == AssetCategory.Decoration)
        {
            return name switch
            {
                "marketstand_1" or "cart" or "gazebo" or "well" => 0.0115f,
                "fence" => 0.012f,
                _ => 0.0105f,
            };
        }

        return currentScale;
    }

    static float? ResolveTargetHeight(AssetCategory category, string name, float? currentTarget)
    {
        if (currentTarget.HasValue)
        {
            return currentTarget;
        }

        return category switch
        {
            AssetCategory.Humanoid => 1.8f,
            AssetCategory.Animal => name switch
            {
                "horse" => 1.85f,
                "stag" => 1.75f,
                "deer" => 1.45f,
                "bull" => 1.8f,
                "cow" => 1.65f,
                "wolf" => 0.95f,
                "fox" => 0.65f,
                "husky" => 0.85f,
                _ => 1.0f,
            },
            AssetCategory.Building => name switch
            {
                "bell_tower" => 18.0f,
                "mill" => 12.0f,
                "sawmill" => 10.0f,
                "stable" => 8.0f,
                "inn" => 7.5f,
                "blacksmith" => 6.5f,
                "house_1" or "house_2" or "house_3" or "house_4" => 5.75f,
                _ => 6.0f,
            },
            AssetCategory.Prop => name switch
            {
                "marketstand_1" => 3.0f,
                "cart" => 1.7f,
                "bench_1" => 1.0f,
                "barrel" => 0.95f,
                "crate" => 0.9f,
                "fence" => 1.2f,
                "well" => 1.8f,
                "gazebo" => 4.2f,
                "bonfire" => 0.8f,
                "rock_1" or "rock_2" or "rock_3" => 1.25f,
                "hay" => 1.0f,
                "bag" => 0.6f,
                "cauldron" => 0.85f,
                _ => 1.0f,
            },
            AssetCategory.Decoration => name switch
            {
                "fence" => 1.2f,
                "bonfire" => 0.8f,
                "rock_1" or "rock_2" or "rock_3" => 1.1f,
                "hay" => 1.0f,
                "barrel" => 0.95f,
                "crate" => 0.9f,
                "bag" => 0.6f,
                "cauldron" => 0.85f,
                _ => 0.9f,
            },
            _ => null,
        };
    }

    static float ResolveGroundBias(AssetCategory category, string name, float currentBias)
    {
        if (category == AssetCategory.Animal)
        {
            return name switch
            {
                "horse" or "stag" or "deer" => 0.08f,
                "bull" or "cow" => 0.07f,
                _ => 0.06f,
            };
        }

        if (category == AssetCategory.Building)
        {
            return 0.01f;
        }

        return currentBias;
    }

    static List<AssetProfile> BuildHumanoidProfiles(SandboxOptions options, string localCharacter)
    {
        string[] idle = { "Idle_Loop", "Idle" };
        string[] walk = { "Walk_Loop", "Walk_Formal_Loop", "Jog_Fwd_Loop" };
        string[] run = { "Sprint_Loop", "Jog_Fwd_Loop", "Walk_Loop" };

        return DistinctExistingPaths(
                new[]
                {
                    localCharacter,
                    Path.Combine(options.ThroneAssetsRoot, "characters", "Character.glb"),
                    Path.Combine(options.ThroneAssetsRoot, "characters", "humanoid.glb"),
                }
            )
            .Select(path => DynamicProfile(path, AssetCategory.Humanoid, 1f, idle, walk, run))
            .ToList();
    }

    static List<AssetProfile> BuildAnimalProfiles(SandboxOptions options)
    {
        string[] idle = { "Idle", "Idle_2", "Idle_HeadLow", "Eating" };
        string[] walk = { "Walk", "Trot" };
        string[] run = { "Gallop", "Run", "Sprint", "Jog" };

        return EnumeratePreferredModels(
                Path.Combine(options.ThroneAssetsRoot, "animals"),
                "horse.fbx",
                "deer.fbx",
                "wolf.fbx",
                "fox.fbx",
                "cow.fbx",
                "husky.fbx",
                "stag.fbx",
                "bull.fbx"
            )
            .Select(path => DynamicProfile(path, AssetCategory.Animal, 0.01f, idle, walk, run))
            .ToList();
    }

    static List<AssetProfile> BuildStaticProfiles(
        SandboxOptions options,
        string subDirectory,
        AssetCategory category,
        params string[] fileNames
    )
    {
        return EnumeratePreferredModels(Path.Combine(options.ThroneAssetsRoot, subDirectory), fileNames)
            .Select(path => StaticProfile(path, category, 0.01f))
            .ToList();
    }

    static ScenarioState BuildScenario(GoudGame game, SandboxOptions options, uint sceneId, Random rng)
    {
        string baseDir = AppDomain.CurrentDomain.BaseDirectory;
        string localCharacter = Path.Combine(baseDir, "assets", "Character.glb");
        List<AssetProfile> playerProfiles = BuildHumanoidProfiles(options, localCharacter);
        List<AssetProfile> humanoidProfiles = FilterProfiles(playerProfiles, options, includeHumanoids: true);
        List<AssetProfile> animalProfiles = FilterProfiles(BuildAnimalProfiles(options), options);
        List<AssetProfile> buildingProfiles = FilterProfiles(BuildStaticProfiles(
            options,
            "buildings",
            AssetCategory.Building,
            "Inn.fbx",
            "House_1.fbx",
            "House_2.fbx",
            "House_3.fbx",
            "House_4.fbx",
            "Blacksmith.fbx",
            "Mill.fbx",
            "Sawmill.fbx",
            "Stable.fbx",
            "Bell_Tower.fbx"
        ), options);
        List<AssetProfile> propProfiles = FilterProfiles(BuildStaticProfiles(
            options,
            "props",
            AssetCategory.Prop,
            "MarketStand_1.fbx",
            "Cart.fbx",
            "Bench_1.fbx",
            "Barrel.fbx",
            "Crate.fbx",
            "Fence.fbx",
            "Well.fbx",
            "Gazebo.fbx",
            "Bonfire.fbx",
            "Rock_1.fbx",
            "Rock_2.fbx",
            "Rock_3.fbx",
            "Hay.fbx",
            "Bag.fbx",
            "Cauldron.fbx"
        ), options);
        List<AssetProfile> decorationProfiles = FilterProfiles(BuildStaticProfiles(
            options,
            "props",
            AssetCategory.Decoration,
            "Barrel.fbx",
            "Crate.fbx",
            "Fence.fbx",
            "Rock_1.fbx",
            "Rock_2.fbx",
            "Rock_3.fbx",
            "Hay.fbx",
            "Bag.fbx",
            "Cauldron.fbx",
            "Bonfire.fbx"
        ), options);

        RigConfig? playerRig = null;
        List<uint> sourceModelIds = new();
        List<string> loadedAssets = new();
        List<string> failedAssets = new();
        List<AssetDiagnostics> assetDiagnostics = new();

        foreach (AssetProfile profile in playerProfiles)
        {
            playerRig = LoadRig(
                game,
                profile,
                sourceModelIds,
                loadedAssets,
                failedAssets,
                new Dictionary<uint, AssetProfile>(),
                assetDiagnostics
            );
            if (playerRig is not null)
            {
                break;
            }
        }

        if (playerRig is null)
        {
            throw new InvalidOperationException(
                "No humanoid model could be loaded. Checked local sandbox assets and Throne character assets."
            );
        }

        playerX = 0f;
        playerY = playerRig.GroundOffsetY;
        playerZ = InitialPlayerZ(options.Scenario);
        playerFacing = 0f;
        currentAnim = AnimState.Idle;

        game.SetModelPosition(playerRig.SourceModelId, playerX, playerRig.GroundOffsetY, playerZ);
        game.SetModelRotation(
            playerRig.SourceModelId,
            playerRig.RotationX,
            playerFacing + playerRig.RotationY,
            playerRig.RotationZ
        );
        game.SetModelScale(playerRig.SourceModelId, playerRig.Scale, playerRig.Scale, playerRig.Scale);
        game.AddModelToScene(sceneId, playerRig.SourceModelId);
        if (playerRig.AnimCount > 0)
        {
            game.PlayAnimation(playerRig.SourceModelId, playerRig.IdleAnim, true);
        }

        ScenarioState state = new()
        {
            PlayerRig = playerRig,
            PlayerModelId = playerRig.SourceModelId,
        };

        state.SourceModelIds.AddRange(sourceModelIds);
        state.LoadedAssets.AddRange(loadedAssets);
        state.FailedAssets.AddRange(failedAssets);
        state.AssetDiagnostics.AddRange(assetDiagnostics);
        state.SourceProfiles[playerRig.SourceModelId] = playerProfiles.First(p => p.Path == playerRig.Path);
        state.HumanoidRigs.Add(playerRig);

        bool spawnHumanoids = ShouldSpawnHumanoids(options);
        bool spawnAnimals = ShouldSpawnAnimals(options);
        bool spawnStatics = ShouldSpawnStatics(options);

        if (spawnHumanoids && options.Scenario != ScenarioKind.Baseline)
        {
            foreach (AssetProfile profile in humanoidProfiles)
            {
                if (profile.Path == playerRig.Path)
                {
                    continue;
                }
                RigConfig? extraHumanoid = LoadRig(
                    game,
                    profile,
                    state.SourceModelIds,
                    state.LoadedAssets,
                    state.FailedAssets,
                    state.SourceProfiles,
                    state.AssetDiagnostics
                );
                if (extraHumanoid is not null && extraHumanoid.Path != playerRig.Path)
                {
                    state.HumanoidRigs.Add(extraHumanoid);
                }
            }
        }

        if (spawnAnimals)
        {
            int animalProfileLimit = AnimalProfileLimit(options, animalProfiles.Count);
            foreach (AssetProfile animalProfile in animalProfiles.Take(animalProfileLimit))
            {
                RigConfig? animalRig = LoadRig(
                    game,
                    animalProfile,
                    state.SourceModelIds,
                    state.LoadedAssets,
                    state.FailedAssets,
                    state.SourceProfiles,
                    state.AssetDiagnostics
                );
                if (animalRig is not null)
                {
                    state.AnimalRigs.Add(animalRig);
                }
            }
        }

        int humanoidCount = options.InitialNpcCount;
        int animalCount = options.Scenario switch
        {
            ScenarioKind.FbxAnimalDiagnostic => animalProfiles.Count == 0 ? 0 : Math.Max(24, options.InitialNpcCount / 2),
            ScenarioKind.MixedCorrectnessDiagnostic => Math.Max(18, options.InitialNpcCount / 3),
            ScenarioKind.FullGameStress => Math.Max(48, options.InitialNpcCount / 2),
            ScenarioKind.GameLikeVillage => Math.Max(24, options.InitialNpcCount / 3),
            ScenarioKind.ThroneCurrentMix => Math.Max(16, options.InitialNpcCount / 2),
            ScenarioKind.WorstCaseAllVisible => Math.Max(24, options.InitialNpcCount / 2),
            ScenarioKind.SpawnDespawnChurn => Math.Max(12, options.InitialNpcCount / 3),
            _ => 0,
        };
        if (!spawnHumanoids)
        {
            humanoidCount = 0;
        }
        if (!spawnAnimals)
        {
            animalCount = 0;
        }

        for (int i = 0; i < humanoidCount; i++)
        {
            RigConfig rig = state.HumanoidRigs[i % state.HumanoidRigs.Count];
            CrowdAgent agent = CreateCrowdAgent(
                game,
                rig,
                sceneId,
                rng,
                LayoutForScenario(options.Scenario),
                i
            );
            if (agent.ModelId != 0)
            {
                state.Agents.Add(agent);
            }
        }

        for (int i = 0; i < animalCount && state.AnimalRigs.Count > 0; i++)
        {
            RigConfig rig = state.AnimalRigs[i % state.AnimalRigs.Count];
            CrowdAgent agent = CreateCrowdAgent(
                game,
                rig,
                sceneId,
                rng,
                LayoutForScenario(options.Scenario),
                i
            );
            if (agent.ModelId != 0)
            {
                state.Agents.Add(agent);
            }
        }

        if (spawnStatics)
        {
            List<uint> buildingSources = LoadStaticSources(
                game,
                buildingProfiles,
                options.Scenario == ScenarioKind.FullGameStress ? buildingProfiles.Count : (options.Scenario == ScenarioKind.GameLikeVillage ? 8 : 6),
                state.SourceModelIds,
                state.LoadedAssets,
                state.FailedAssets,
                state.SourceProfiles,
                state.AssetDiagnostics
            );

            List<uint> propSources = LoadStaticSources(
                game,
                propProfiles,
                options.Scenario == ScenarioKind.FullGameStress ? propProfiles.Count : (options.Scenario == ScenarioKind.GameLikeVillage ? 8 : 10),
                state.SourceModelIds,
                state.LoadedAssets,
                state.FailedAssets,
                state.SourceProfiles,
                state.AssetDiagnostics
            );

            List<uint> decorationSources = LoadStaticSources(
                game,
                decorationProfiles,
                options.Scenario == ScenarioKind.FullGameStress ? decorationProfiles.Count : 10,
                state.SourceModelIds,
                state.LoadedAssets,
                state.FailedAssets,
                state.SourceProfiles,
                state.AssetDiagnostics
            );

            int buildingInstances = options.Scenario switch
            {
                ScenarioKind.FbxStaticDiagnostic => 18,
                ScenarioKind.MixedCorrectnessDiagnostic => 18,
                ScenarioKind.FullGameStress => 32,
                ScenarioKind.GameLikeVillage => 24,
                ScenarioKind.WorstCaseAllVisible => 18,
                ScenarioKind.SpawnDespawnChurn => 14,
                _ => 14,
            };
            int propInstances = options.Scenario switch
            {
                ScenarioKind.FbxStaticDiagnostic => 64,
                ScenarioKind.MixedCorrectnessDiagnostic => 64,
                ScenarioKind.FullGameStress => 96,
                ScenarioKind.GameLikeVillage => 72,
                ScenarioKind.WorstCaseAllVisible => 64,
                ScenarioKind.SpawnDespawnChurn => 48,
                _ => 48,
            };
            int decorationInstances = options.Scenario switch
            {
                ScenarioKind.FbxStaticDiagnostic => 48,
                ScenarioKind.MixedCorrectnessDiagnostic => 48,
                ScenarioKind.FullGameStress => 128,
                ScenarioKind.GameLikeVillage => 96,
                ScenarioKind.WorstCaseAllVisible => 72,
                ScenarioKind.SpawnDespawnChurn => 40,
                _ => 0,
            };

            state.StaticBuildingCount = ScatterStaticInstances(
                game,
                sceneId,
                buildingSources,
                buildingInstances,
                rng,
                state.StaticInstances,
                state.SourceProfiles,
                LayoutForScenario(options.Scenario),
                StaticScatterKind.Building
            );
            state.StaticPropCount = ScatterStaticInstances(
                game,
                sceneId,
                propSources,
                propInstances,
                rng,
                state.StaticInstances,
                state.SourceProfiles,
                LayoutForScenario(options.Scenario),
                StaticScatterKind.Prop
            );
            state.StaticDecorationCount = ScatterStaticInstances(
                game,
                sceneId,
                decorationSources,
                decorationInstances,
                rng,
                state.StaticInstances,
                state.SourceProfiles,
                LayoutForScenario(options.Scenario),
                StaticScatterKind.Decoration
            );
        }

        return state;
    }

    static float InitialPlayerZ(ScenarioKind scenario)
    {
        return scenario switch
        {
            ScenarioKind.FullGameStress => -8f,
            ScenarioKind.FbxStaticDiagnostic => 18f,
            ScenarioKind.FbxAnimalDiagnostic => 18f,
            ScenarioKind.MixedCorrectnessDiagnostic => 18f,
            _ => 0f,
        };
    }

    static List<AssetProfile> FilterProfiles(
        IEnumerable<AssetProfile> profiles,
        SandboxOptions options,
        bool includeHumanoids = false
    )
    {
        List<AssetProfile> filtered = new();
        foreach (AssetProfile profile in profiles)
        {
            if (!ProfileMatchesFilters(profile, options, includeHumanoids))
            {
                continue;
            }
            filtered.Add(profile);
        }
        return filtered;
    }

    static bool ProfileMatchesFilters(AssetProfile profile, SandboxOptions options, bool includeHumanoids)
    {
        if (options.OnlyCategory.HasValue && profile.Category != options.OnlyCategory.Value)
        {
            return false;
        }

        if (!string.IsNullOrWhiteSpace(options.OnlyAsset))
        {
            string requested = options.OnlyAsset!;
            string assetName = Path.GetFileNameWithoutExtension(profile.Path);
            string fileName = Path.GetFileName(profile.Path);
            if (!assetName.Equals(requested, StringComparison.OrdinalIgnoreCase) &&
                !fileName.Equals(requested, StringComparison.OrdinalIgnoreCase))
            {
                return false;
            }
        }

        if (options.NoHumanoids && profile.Category == AssetCategory.Humanoid && !includeHumanoids)
        {
            return false;
        }
        if (options.NoAnimals && profile.Category == AssetCategory.Animal)
        {
            return false;
        }
        if (options.NoStatics && (profile.Category == AssetCategory.Building ||
                                  profile.Category == AssetCategory.Prop ||
                                  profile.Category == AssetCategory.Decoration))
        {
            return false;
        }

        return options.Scenario switch
        {
            ScenarioKind.FbxStaticDiagnostic => profile.Category is AssetCategory.Building or AssetCategory.Prop or AssetCategory.Decoration,
            ScenarioKind.FbxAnimalDiagnostic => profile.Category == AssetCategory.Animal,
            ScenarioKind.MixedCorrectnessDiagnostic => profile.Category is AssetCategory.Humanoid or AssetCategory.Animal or AssetCategory.Building or AssetCategory.Prop or AssetCategory.Decoration,
            _ => true,
        };
    }

    static bool ShouldSpawnHumanoids(SandboxOptions options)
    {
        if (options.NoHumanoids)
        {
            return false;
        }

        return options.Scenario switch
        {
            ScenarioKind.FbxStaticDiagnostic => false,
            ScenarioKind.FbxAnimalDiagnostic => false,
            _ => true,
        };
    }

    static bool ShouldSpawnAnimals(SandboxOptions options)
    {
        if (options.NoAnimals)
        {
            return false;
        }

        return options.Scenario switch
        {
            ScenarioKind.Baseline => false,
            ScenarioKind.FbxStaticDiagnostic => false,
            _ => true,
        };
    }

    static bool ShouldSpawnStatics(SandboxOptions options)
    {
        if (options.NoStatics)
        {
            return false;
        }

        return options.Scenario switch
        {
            ScenarioKind.Baseline => false,
            ScenarioKind.VariedAnimationCrowd => false,
            ScenarioKind.FbxAnimalDiagnostic => false,
            _ => true,
        };
    }

    static int AnimalProfileLimit(SandboxOptions options, int availableCount)
    {
        return options.Scenario switch
        {
            ScenarioKind.FbxAnimalDiagnostic => availableCount,
            ScenarioKind.MixedCorrectnessDiagnostic => Math.Min(availableCount, 6),
            ScenarioKind.FullGameStress => availableCount,
            ScenarioKind.GameLikeVillage => Math.Min(availableCount, 8),
            _ => Math.Min(availableCount, 6),
        };
    }

    static RigConfig PickSpawnRig(ScenarioState state, int index)
    {
        if (state.AnimalRigs.Count == 0)
        {
            return state.HumanoidRigs[index % state.HumanoidRigs.Count];
        }

        int total = state.HumanoidRigs.Count + state.AnimalRigs.Count;
        int cursor = index % total;
        if (cursor < state.HumanoidRigs.Count)
        {
            return state.HumanoidRigs[cursor];
        }

        return state.AnimalRigs[cursor - state.HumanoidRigs.Count];
    }

    static CrowdAgent CreateCrowdAgent(
        GoudGame game,
        RigConfig rig,
        uint sceneId,
        Random rng,
        SpawnLayout layout,
        int spawnIndex
    )
    {
        CrowdAgent agent = new();
        agent.ModelId = game.InstantiateModel(rig.SourceModelId);
        if (agent.ModelId == 0)
        {
            return agent;
        }

        (agent.X, agent.Z) = PickSpawnPoint(rng, layout, rig.IsAnimal, spawnIndex);
        (agent.TargetX, agent.TargetZ) = PickTarget(rng, layout, rig.IsAnimal);
        if (MathF.Abs(agent.TargetX - agent.X) < 2.0f && MathF.Abs(agent.TargetZ - agent.Z) < 2.0f)
        {
            (agent.TargetX, agent.TargetZ) = PickTarget(rng, layout, rig.IsAnimal);
        }
        agent.Facing = (float)(rng.NextDouble() * 360.0);
        agent.IsMoving = rng.NextDouble() > 0.35;
        agent.IdleTimer = agent.IsMoving ? 0f : (float)(rng.NextDouble() * 2.0 + 0.75);
        agent.MoveSpeed = rig.MoveSpeed;
        agent.SpeedMultiplier = 0.75f + (float)(rng.NextDouble() * 0.55);
        agent.IdleAnim = rig.AnimCount > 0 ? rig.IdleAnim : -1;
        agent.WalkAnim = rig.AnimCount > 0 ? rig.WalkAnim : -1;
        agent.RunAnim = rig.AnimCount > 0 ? rig.RunAnim : -1;
        agent.IsAnimal = rig.IsAnimal;
        agent.Y = rig.GroundOffsetY;
        agent.RotationX = rig.RotationX;
        agent.YawOffset = rig.RotationY;
        agent.RotationZ = rig.RotationZ;

        game.SetModelPosition(agent.ModelId, agent.X, agent.Y, agent.Z);
        game.SetModelRotation(
            agent.ModelId,
            agent.RotationX,
            agent.Facing + agent.YawOffset,
            agent.RotationZ
        );
        game.SetModelScale(agent.ModelId, rig.Scale, rig.Scale, rig.Scale);
        game.AddModelToScene(sceneId, agent.ModelId);

        if (rig.AnimCount > 0)
        {
            int startAnim = agent.IsMoving ? rig.WalkAnim : rig.IdleAnim;
            game.PlayAnimation(agent.ModelId, Math.Max(0, startAnim), true);
            game.SetAnimationSpeed(agent.ModelId, agent.SpeedMultiplier);
        }

        return agent;
    }

    static RigConfig? LoadRig(
        GoudGame game,
        AssetProfile profile,
        List<uint> sourceModelIds,
        List<string> loadedAssets,
        List<string> failedAssets,
        Dictionary<uint, AssetProfile> sourceProfiles,
        List<AssetDiagnostics> assetDiagnostics
    )
    {
        string path = profile.Path;
        if (!File.Exists(path))
        {
            failedAssets.Add(path);
            return null;
        }

        uint modelId = game.LoadModel(path);
        if (modelId == 0)
        {
            failedAssets.Add(path);
            Console.WriteLine($"Failed to load model: {path}");
            return null;
        }

        int animCount = game.GetAnimationCount(modelId);
        (int idleAnim, int walkAnim, int runAnim) = ResolveAnimationIndices(game, modelId, animCount, profile);
        string[] animationNames = Enumerable.Range(0, animCount)
            .Select(index => game.GetAnimationName(modelId, index))
            .ToArray();
        BoundingBox3D bounds = game.GetModelBoundingBox(modelId);
        float finalScale = ResolveFinalScale(bounds, profile.Scale, profile.TargetHeight);
        float groundOffsetY = ComputeGroundOffsetY(bounds, finalScale, profile.GroundOffsetBias);

        sourceModelIds.Add(modelId);
        loadedAssets.Add(path);
        sourceProfiles[modelId] = profile;
        AppendAssetDiagnostics(assetDiagnostics, profile, bounds, finalScale, groundOffsetY, animCount, idleAnim, walkAnim, runAnim, animationNames);

        string name = Path.GetFileNameWithoutExtension(path);
        return new RigConfig
        {
            Name = name,
            Path = path,
            SourceModelId = modelId,
            AnimCount = animCount,
            IdleAnim = idleAnim,
            WalkAnim = walkAnim,
            RunAnim = runAnim,
            Scale = finalScale,
            MoveSpeed = GuessMoveSpeed(name, profile.Category == AssetCategory.Animal),
            IsAnimal = profile.Category == AssetCategory.Animal,
            GroundOffsetY = groundOffsetY,
            RotationX = profile.RotationX,
            RotationY = profile.RotationY,
            RotationZ = profile.RotationZ,
        };
    }

    static List<uint> LoadStaticSources(
        GoudGame game,
        IEnumerable<AssetProfile> profiles,
        int maxCount,
        List<uint> sourceModelIds,
        List<string> loadedAssets,
        List<string> failedAssets,
        Dictionary<uint, AssetProfile> sourceProfiles,
        List<AssetDiagnostics> assetDiagnostics
    )
    {
        List<uint> loaded = new();

        foreach (AssetProfile profile in profiles.Take(maxCount))
        {
            string path = profile.Path;
            if (!File.Exists(path))
            {
                failedAssets.Add(path);
                continue;
            }

            uint modelId = game.LoadModel(path);
            if (modelId == 0)
            {
                failedAssets.Add(path);
                Console.WriteLine($"Failed to load static model: {path}");
                continue;
            }

            sourceModelIds.Add(modelId);
            loadedAssets.Add(path);
            sourceProfiles[modelId] = profile;
            BoundingBox3D bounds = game.GetModelBoundingBox(modelId);
            float finalScale = ResolveFinalScale(bounds, profile.Scale, profile.TargetHeight);
            float groundOffsetY = ComputeGroundOffsetY(bounds, finalScale, profile.GroundOffsetBias);
            AppendAssetDiagnostics(assetDiagnostics, profile, bounds, finalScale, groundOffsetY, 0, -1, -1, -1, Array.Empty<string>());
            loaded.Add(modelId);
        }

        return loaded;
    }

    static int ScatterStaticInstances(
        GoudGame game,
        uint sceneId,
        List<uint> sourceModels,
        int count,
        Random rng,
        List<uint> staticInstances,
        Dictionary<uint, AssetProfile> sourceProfiles,
        SpawnLayout layout,
        StaticScatterKind scatterKind
    )
    {
        int created = 0;
        if (sourceModels.Count == 0)
        {
            return created;
        }

        for (int i = 0; i < count; i++)
        {
            uint sourceModel = sourceModels[i % sourceModels.Count];
            uint modelId = game.InstantiateModel(sourceModel);
            if (modelId == 0)
            {
                continue;
            }

            AssetProfile profile = sourceProfiles[sourceModel];
            float x;
            float z;
            float yaw;
            if (layout == SpawnLayout.Village)
            {
                (x, z, yaw) = PickVillageStaticAnchor(i, scatterKind, rng);
            }
            else if (layout == SpawnLayout.Compact)
            {
                x = -22f + (i % 6) * 8f + (float)(rng.NextDouble() * 1.5 - 0.75);
                z = 10f + (i / 6) * 7f + (float)(rng.NextDouble() * 1.5 - 0.75);
                yaw = (i % 2 == 0) ? 0f : 90f;
            }
            else
            {
                x = (float)(rng.NextDouble() * 150.0 - 75.0);
                z = (float)(rng.NextDouble() * 150.0 - 75.0);
                yaw = (float)(rng.NextDouble() * 360.0);
            }
            BoundingBox3D bounds = game.GetModelBoundingBox(sourceModel);
            float baseScale = ResolveFinalScale(bounds, profile.Scale, profile.TargetHeight);
            float scale = baseScale * (0.95f + (float)(rng.NextDouble() * 0.1));
            float groundOffsetY = ComputeGroundOffsetY(bounds, scale, profile.GroundOffsetBias);

            game.SetModelPosition(modelId, x, groundOffsetY, z);
            game.SetModelRotation(
                modelId,
                profile.RotationX,
                yaw + profile.RotationY,
                profile.RotationZ
            );
            game.SetModelScale(modelId, scale, scale, scale);
            game.SetModelStatic(modelId, true);
            game.AddModelToScene(sceneId, modelId);

            staticInstances.Add(modelId);
            created++;
        }

        return created;
    }

    static (float x, float z) PickSpawnPoint(
        Random rng,
        SpawnLayout layout,
        bool isAnimal,
        int spawnIndex
    )
    {
        if (layout != SpawnLayout.Village)
        {
            return PickTarget(rng, layout, isAnimal);
        }

        if (isAnimal)
        {
            int penIndex = spawnIndex % 2;
            int slot = spawnIndex / 2;
            int column = slot % 4;
            int row = slot / 4;
            float baseX = penIndex == 0 ? (-22f + column * 3.8f) : (10f + column * 3.8f);
            float baseZ = 8f + row * 4.6f;
            float jitterX = (float)(rng.NextDouble() * 0.8 - 0.4);
            float jitterZ = (float)(rng.NextDouble() * 0.8 - 0.4);
            return (baseX + jitterX, baseZ + jitterZ);
        }

        int lane = spawnIndex % 5;
        int rowIndex = spawnIndex / 5;
        float[] laneX = { -10f, -5f, 0f, 5f, 10f };
        float baseHumanoidX = laneX[lane];
        float baseHumanoidZ = 26f + rowIndex * 4.5f;
        float humanoidJitterX = (float)(rng.NextDouble() * 1.2 - 0.6);
        float humanoidJitterZ = (float)(rng.NextDouble() * 1.2 - 0.6);
        return (baseHumanoidX + humanoidJitterX, baseHumanoidZ + humanoidJitterZ);
    }

    static (float x, float z) PickTarget(Random rng, SpawnLayout layout, bool isAnimal = false)
    {
        if (layout == SpawnLayout.Compact)
        {
            float x = (float)(rng.NextDouble() * 48.0 - 24.0);
            float z = (float)(rng.NextDouble() * 28.0 + 8.0);
            return (x, z);
        }
        if (layout == SpawnLayout.Village)
        {
            (float baseX, float baseZ)[] waypoints = isAnimal
                ? new (float, float)[]
                {
                    (-24f, 10f), (-20f, 18f), (-16f, 26f), (-12f, 14f),
                    (12f, 14f), (16f, 26f), (20f, 18f), (24f, 10f),
                }
                : new (float, float)[]
                {
                    (-10f, 26f), (-6f, 32f), (-2f, 38f), (2f, 30f),
                    (6f, 36f), (10f, 26f), (-8f, 44f), (0f, 42f),
                    (8f, 44f), (14f, 36f), (-14f, 36f), (0f, 50f),
                };
            (float baseX, float baseZ) anchor = waypoints[rng.Next(waypoints.Length)];
            float jitterX = (float)(rng.NextDouble() * (isAnimal ? 6.0 : 4.0) - (isAnimal ? 3.0 : 2.0));
            float jitterZ = (float)(rng.NextDouble() * (isAnimal ? 6.0 : 4.0) - (isAnimal ? 3.0 : 2.0));
            return (anchor.baseX + jitterX, anchor.baseZ + jitterZ);
        }

        float wideX = (float)(rng.NextDouble() * (npcBoundsMax - npcBoundsMin) + npcBoundsMin);
        float wideZ = (float)(rng.NextDouble() * (npcBoundsMax - npcBoundsMin) + npcBoundsMin);
        return (wideX, wideZ);
    }

    static (float x, float z, float yaw) PickVillageStaticAnchor(
        int index,
        StaticScatterKind scatterKind,
        Random rng
    )
    {
        return scatterKind switch
        {
            StaticScatterKind.Building => PickBuildingAnchor(index, rng),
            StaticScatterKind.Prop => PickPropAnchor(index, rng),
            _ => PickDecorationAnchor(index, rng),
        };
    }

    static (float x, float z, float yaw) PickBuildingAnchor(int index, Random rng)
    {
        int column = index % 4;
        int row = index / 4;
        float[] laneX = { -28f, -10f, 10f, 28f };
        float[] yawByLane = { 90f, 90f, 270f, 270f };
        float baseX = laneX[column];
        float baseZ = 10f + row * 14f;
        float jitterX = (float)(rng.NextDouble() * 0.6 - 0.3);
        float jitterZ = (float)(rng.NextDouble() * 1.0 - 0.5);
        return (baseX + jitterX, baseZ + jitterZ, yawByLane[column]);
    }

    static (float x, float z, float yaw) PickPropAnchor(int index, Random rng)
    {
        int district = index % 4;
        int slot = index / 4;
        int column = slot % 4;
        int row = slot / 4;

        float districtCenterX = district switch
        {
            0 => -18f,
            1 => -6f,
            2 => 6f,
            _ => 18f,
        };
        float districtCenterZ = district < 2 ? 12f : 24f;
        float baseX = districtCenterX + (column - 1.5f) * 3.5f;
        float baseZ = districtCenterZ + row * 5.0f;
        float jitterX = (float)(rng.NextDouble() * 1.0 - 0.5);
        float jitterZ = (float)(rng.NextDouble() * 1.0 - 0.5);
        float yaw = (index * 37) % 360;
        return (baseX + jitterX, baseZ + jitterZ, yaw);
    }

    static (float x, float z, float yaw) PickDecorationAnchor(int index, Random rng)
    {
        int band = index % 6;
        int slot = index / 6;
        int row = slot / 3;
        int column = slot % 3;

        float baseX = band switch
        {
            0 => -30f + column * 4f,
            1 => -16f + column * 4f,
            2 => -2f + column * 4f,
            3 => 12f + column * 4f,
            4 => -22f + column * 4f,
            _ => 16f + column * 4f,
        };
        float baseZ = band < 4 ? (8f + row * 6f) : (20f + row * 6f);
        float jitterX = (float)(rng.NextDouble() * 0.8 - 0.4);
        float jitterZ = (float)(rng.NextDouble() * 0.8 - 0.4);
        float yaw = (index * 53) % 360;
        return (baseX + jitterX, baseZ + jitterZ, yaw);
    }

    static float WrapDegrees(float degrees)
    {
        while (degrees > 180f)
        {
            degrees -= 360f;
        }
        while (degrees < -180f)
        {
            degrees += 360f;
        }
        return degrees;
    }

    static (int idle, int walk, int run) ResolveAnimationIndices(
        GoudGame game,
        uint modelId,
        int animCount,
        AssetProfile profile
    )
    {
        if (animCount <= 0)
        {
            return (-1, -1, -1);
        }

        int idle = FindAnimationIndexByNames(game, modelId, animCount, profile.IdleClipNames, 0);
        int walk = FindAnimationIndexByNames(game, modelId, animCount, profile.WalkClipNames, Math.Min(1, animCount - 1));
        int run = FindAnimationIndexByNames(game, modelId, animCount, profile.RunClipNames, walk >= 0 ? walk : Math.Min(2, animCount - 1));
        return (idle, walk, run);
    }

    static int FindAnimationIndexByNames(
        GoudGame game,
        uint modelId,
        int animCount,
        string[] preferredNames,
        int fallbackIndex
    )
    {
        Dictionary<string, int> animations = new(StringComparer.OrdinalIgnoreCase);
        for (int i = 0; i < animCount; i++)
        {
            animations[NormalizeAnimName(game.GetAnimationName(modelId, i))] = i;
        }

        foreach (string preferredName in preferredNames)
        {
            string normalized = NormalizeAnimName(preferredName);
            if (animations.TryGetValue(normalized, out int exact))
            {
                return exact;
            }

            KeyValuePair<string, int> partialMatch = animations.FirstOrDefault(
                entry => entry.Key.Contains(normalized, StringComparison.OrdinalIgnoreCase)
            );
            if (!string.IsNullOrEmpty(partialMatch.Key))
            {
                return partialMatch.Value;
            }
        }

        return fallbackIndex;
    }

    static string NormalizeAnimName(string raw)
    {
        return raw
            .Split('|')
            .Last()
            .Replace(" ", "")
            .Replace("-", "")
            .Replace("_", "")
            .ToLowerInvariant();
    }

    static float ResolveFinalScale(BoundingBox3D bounds, float fallbackScale, float? targetHeight)
    {
        if (targetHeight is null)
        {
            return fallbackScale;
        }

        float rawHeight = MathF.Max(bounds.MaxY - bounds.MinY, 0.0001f);
        float fittedScale = targetHeight.Value / rawHeight;
        if (!float.IsFinite(fittedScale) || fittedScale <= 0.0f)
        {
            return fallbackScale;
        }

        return fittedScale;
    }

    static float ComputeGroundOffsetY(BoundingBox3D bounds, float scale, float bias)
    {
        return (-bounds.MinY * scale) + bias;
    }

    static void AppendAssetDiagnostics(
        List<AssetDiagnostics> diagnostics,
        AssetProfile profile,
        BoundingBox3D bounds,
        float finalScale,
        float groundOffsetY,
        int animationCount,
        int idleAnim,
        int walkAnim,
        int runAnim,
        string[] animationNames)
    {
        diagnostics.Add(new AssetDiagnostics
        {
            Name = profile.Name,
            Path = profile.Path,
            Category = profile.Category.ToString(),
            IsDynamic = profile.IsDynamic,
            RawWidth = bounds.MaxX - bounds.MinX,
            RawHeight = bounds.MaxY - bounds.MinY,
            RawDepth = bounds.MaxZ - bounds.MinZ,
            FinalScale = finalScale,
            GroundOffsetY = groundOffsetY,
            AnimationCount = animationCount,
            IdleAnimation = idleAnim,
            WalkAnimation = walkAnim,
            RunAnimation = runAnim,
            AnimationNames = animationNames,
        });
    }

    static float GuessMoveSpeed(string name, bool isAnimal)
    {
        string lower = name.ToLowerInvariant();
        if (!isAnimal)
        {
            return 3f;
        }

        if (lower.Contains("horse") || lower.Contains("stag") || lower.Contains("deer"))
        {
            return 4.5f;
        }
        if (lower.Contains("wolf") || lower.Contains("fox") || lower.Contains("husky"))
        {
            return 4.0f;
        }
        if (lower.Contains("cow") || lower.Contains("bull"))
        {
            return 2.5f;
        }

        return 3.25f;
    }

    static IEnumerable<string> DistinctExistingPaths(IEnumerable<string> paths)
    {
        HashSet<string> seen = new(StringComparer.OrdinalIgnoreCase);
        foreach (string path in paths)
        {
            if (!seen.Add(path))
            {
                continue;
            }
            if (File.Exists(path))
            {
                yield return path;
            }
        }
    }

    static IEnumerable<string> EnumeratePreferredModels(string directory, params string[] preferredNames)
    {
        HashSet<string> yielded = new(StringComparer.OrdinalIgnoreCase);

        foreach (string name in preferredNames)
        {
            string fullPath = Path.Combine(directory, name);
            if (File.Exists(fullPath) && yielded.Add(fullPath))
            {
                yield return fullPath;
            }
        }

        if (!Directory.Exists(directory))
        {
            yield break;
        }

        foreach (string file in Directory.EnumerateFiles(directory)
                     .Where(IsModelAsset)
                     .OrderBy(Path.GetFileName))
        {
            if (yielded.Add(file))
            {
                yield return file;
            }
        }
    }

    static bool IsModelAsset(string path)
    {
        string ext = Path.GetExtension(path).ToLowerInvariant();
        return ext is ".glb" or ".gltf" or ".fbx" or ".obj";
    }

    static void WriteMetricsReport(
        SandboxOptions options,
        ScenarioState state,
        MetricsAccumulator metrics,
        double elapsedSeconds,
        double lastFps
    )
    {
        if (string.IsNullOrWhiteSpace(options.MetricsOutPath))
        {
            return;
        }

        string outputPath = options.MetricsOutPath!;
        string? outputDir = Path.GetDirectoryName(outputPath);
        if (!string.IsNullOrWhiteSpace(outputDir))
        {
            Directory.CreateDirectory(outputDir);
        }

        List<double> fpsSamples = metrics.FrameMs
            .Where(ms => ms > 0.0)
            .Select(ms => 1000.0 / ms)
            .ToList();

        var report = new
        {
            capturedAtUtc = DateTime.UtcNow.ToString("O"),
            scenario = ScenarioLabel(options.Scenario),
            backend = options.BackendName,
            durationSeconds = elapsedSeconds,
            frameCount = metrics.FrameMs.Count,
            lastFps,
            counts = new
            {
                activeAgents = state.Agents.Count,
                humanoids = state.Agents.Count(agent => !agent.IsAnimal),
                animals = state.Agents.Count(agent => agent.IsAnimal),
                staticBuildings = state.StaticBuildingCount,
                staticProps = state.StaticPropCount,
                staticDecorations = state.StaticDecorationCount,
                staticInstances = state.StaticInstances.Count,
            },
            assets = new
            {
                loaded = state.LoadedAssets.Distinct().OrderBy(path => path).ToArray(),
                failed = state.FailedAssets.Distinct().OrderBy(path => path).ToArray(),
                diagnostics = state.AssetDiagnostics
                    .OrderBy(record => record.Category)
                    .ThenBy(record => record.Name)
                    .ToArray(),
            },
            frameMs = Summarize(metrics.FrameMs),
            fps = Summarize(fpsSamples),
            drawCalls = Summarize(metrics.DrawCalls),
            visibleObjects = Summarize(metrics.VisibleObjects),
            culledObjects = Summarize(metrics.CulledObjects),
            instancedDrawCalls = Summarize(metrics.InstancedDrawCalls),
            activeInstances = Summarize(metrics.ActiveInstances),
            boneMatrixUploads = Summarize(metrics.BoneMatrixUploads),
            animationEvaluations = Summarize(metrics.AnimationEvaluations),
            animationEvaluationsSaved = Summarize(metrics.AnimationEvaluationsSaved),
        };

        string json = JsonSerializer.Serialize(report, new JsonSerializerOptions
        {
            WriteIndented = true,
        });
        File.WriteAllText(outputPath, json);
    }

    static object Summarize(IEnumerable<double> source)
    {
        List<double> values = source
            .Where(value => !double.IsNaN(value) && !double.IsInfinity(value))
            .OrderBy(value => value)
            .ToList();

        if (values.Count == 0)
        {
            return new
            {
                average = 0.0,
                minimum = 0.0,
                maximum = 0.0,
                p50 = 0.0,
                p95 = 0.0,
                p99 = 0.0,
            };
        }

        double Percentile(double pct)
        {
            int index = (int)Math.Ceiling(values.Count * pct) - 1;
            index = Math.Clamp(index, 0, values.Count - 1);
            return values[index];
        }

        return new
        {
            average = values.Average(),
            minimum = values[0],
            maximum = values[^1],
            p50 = Percentile(0.50),
            p95 = Percentile(0.95),
            p99 = Percentile(0.99),
        };
    }
}
