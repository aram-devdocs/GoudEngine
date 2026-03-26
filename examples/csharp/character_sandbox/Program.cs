using System;
using System.Diagnostics;
using GoudEngine;

class Program
{
    static void Main(string[] args)
    {
        var config = GameConfig.Parse(args);
        PrintControls(config);

        using var game = new EngineConfig()
            .SetSize(1280, 720)
            .SetTitle("Character Sandbox")
            .SetRenderBackend(RenderBackendKind.Wgpu)
            .SetWindowBackend(WindowBackendKind.Winit)
            .Build();

        uint sceneId = SceneSetup.Initialize(game);
        var catalog = new AssetCatalog();
        var loader = new ModelLoader(game);

        // Player
        LoadedRig? playerRig = null;
        foreach (var profile in catalog.Humanoids)
        {
            playerRig = loader.LoadRig(profile);
            if (playerRig != null) break;
        }
        if (playerRig == null)
            throw new InvalidOperationException("No humanoid model found. Check assets/Character.glb exists.");

        var player = new PlayerController(game, playerRig, sceneId);

        // Crowd
        var crowd = new CrowdSystem(
            game, loader, catalog, sceneId,
            config.NpcCount, config.AnimalCount, config.Seed,
            config.VariedAnims, config.PhaseLock);

        // Static geometry (buildings + props)
        var rng = new Random(config.Seed + 1000);
        int buildingsPlaced = PlaceStaticModels(game, loader, catalog.Buildings, sceneId, rng, 20);
        int propsPlaced = PlaceStaticModels(game, loader, catalog.Props, sceneId, rng, 30, isBuilding: false);
        Console.WriteLine($"Village: {buildingsPlaced} buildings, {propsPlaced} props");

        // Profile mode: fixed camera, render N frames, dump stats, exit.
        if (config.Profile)
        {
            // Fixed camera position for reproducible screenshots
            game.SetCameraPosition3D(0f, 8f, 15f);
            game.SetCameraRotation3D(-25f, 0f, 0f);

            int profileFrames = 120;
            float totalTime = 0f;
            for (int f = 0; f < profileFrames && !game.ShouldClose(); f++)
            {
                float dt = game.DeltaTime;
                totalTime += dt;
                game.BeginFrame(0.1f, 0.1f, 0.15f, 1.0f);
                crowd.Update(dt);
                game.UpdateAnimations(dt);
                game.Render3D();
                game.EndFrame();
            }

            float avgFps = profileFrames / Math.Max(totalTime, 0.001f);
            int draws = game.GetDrawCalls();
            int visible = game.GetVisibleObjectCount();
            int culled = game.GetCulledObjectCount();
            int instanced = game.GetInstancedDrawCalls();
            int animEval = game.GetAnimationEvaluationCount();
            int animSaved = game.GetAnimationEvaluationSavedCount();

            Console.WriteLine($"PROFILE: FPS={avgFps:F1} Draws={draws} " +
                $"Visible={visible}/{visible + culled} Instanced={instanced} " +
                $"AnimEval={animEval} Saved={animSaved} Agents={crowd.Count}");

            // Write stats to file for automated comparison
            System.IO.File.WriteAllText("/tmp/goud_profile_stats.txt",
                $"fps={avgFps:F1}\ndraws={draws}\ninstanced={instanced}\n" +
                $"agents={crowd.Count}\nanim_eval={animEval}\nanim_saved={animSaved}\n");

            crowd.DestroyAll();
            loader.DestroyAll();
            return;
        }

        // Phase-lock runtime toggle state
        bool phaseLockActive = config.PhaseLock;

        // Stats display
        float fpsTimer = 0f;
        int fpsFrames = 0;

        while (!game.ShouldClose())
        {
            float dt = game.DeltaTime;
            game.BeginFrame(0.1f, 0.1f, 0.15f, 1.0f);

            if (game.IsKeyPressed(Keys.Escape)) { game.Close(); continue; }

            SceneSetup.HandleToggles(game);

            if (game.IsKeyJustPressed(Keys.Equal))
                crowd.SpawnMore(10);
            if (game.IsKeyJustPressed(Keys.Minus))
                crowd.RemoveLast(10);
            if (game.IsKeyJustPressed(Keys.Digit6))
            {
                phaseLockActive = !phaseLockActive;
                crowd.SetPhaseLockAll(phaseLockActive);
                Console.WriteLine($"\nPhase-lock: {(phaseLockActive ? "ON" : "OFF")}");
            }

            player.Update(dt);
            crowd.Update(dt);
            game.UpdateAnimations(dt);

            player.UpdateCamera();
            game.Render3D();
            game.EndFrame();

            // Stats HUD
            fpsTimer += dt;
            fpsFrames++;
            if (fpsTimer >= 1.0f)
            {
                int draws = game.GetDrawCalls();
                int visible = game.GetVisibleObjectCount();
                int culled = game.GetCulledObjectCount();
                int instanced = game.GetInstancedDrawCalls();
                int activeInst = game.GetActiveInstanceCount();
                int animEval = game.GetAnimationEvaluationCount();
                int animSaved = game.GetAnimationEvaluationSavedCount();
                int boneUploads = game.GetBoneMatrixUploadCount();

                Console.Write(
                    $"\rFPS: {fpsFrames / fpsTimer:F1}  Agents: {crowd.Count}  " +
                    $"Draws: {draws}  Visible: {visible}/{visible + culled}  " +
                    $"Instanced: {instanced}  Instances: {activeInst}  " +
                    $"AnimEval: {animEval}  Saved: {animSaved}  Bones: {boneUploads}   "
                );

                fpsFrames = 0;
                fpsTimer = 0f;
            }
        }

        crowd.DestroyAll();
        loader.DestroyAll();

        Console.WriteLine();
        Console.WriteLine("Character Sandbox ended.");
    }

    static int PlaceStaticModels(
        GoudGame game, ModelLoader loader,
        System.Collections.Generic.IReadOnlyList<AssetEntry> entries,
        uint sceneId, Random rng, int count, bool isBuilding = true)
    {
        if (entries.Count == 0) return 0;

        // Load source models
        var sourceIds = new System.Collections.Generic.List<(uint id, AssetEntry entry)>();
        foreach (var entry in entries)
        {
            uint id = loader.LoadStatic(entry);
            if (id != 0) sourceIds.Add((id, entry));
        }
        if (sourceIds.Count == 0) return 0;

        int placed = 0;
        for (int i = 0; i < count; i++)
        {
            var (sourceId, entry) = sourceIds[i % sourceIds.Count];
            uint modelId = game.InstantiateModel(sourceId);
            if (modelId == 0) continue;

            float x, z, yaw;
            if (isBuilding)
                (x, z, yaw) = VillageLayout.PickBuildingPosition(i, rng);
            else
                (x, z, yaw) = VillageLayout.PickPropPosition(i, rng);

            BoundingBox3D bounds = game.GetModelBoundingBox(sourceId);
            // FBX models have height along Z; after -90 X rotation, Z becomes Y.
            float rawH = MathF.Max(bounds.MaxZ - bounds.MinZ, 0.0001f);
            float scale = entry.TargetHeight.HasValue
                ? entry.TargetHeight.Value / rawH
                : entry.FallbackScale;
            if (!float.IsFinite(scale) || scale <= 0f) scale = entry.FallbackScale;
            scale *= (0.95f + (float)(rng.NextDouble() * 0.1));

            // After -90 X rotation, Z-min becomes the ground contact point.
            float groundY = (-bounds.MinZ * scale) + entry.GroundBias;

            game.SetModelPosition(modelId, x, groundY, z);
            game.SetModelRotation(modelId, entry.RotationX, yaw + entry.RotationY, entry.RotationZ);
            game.SetModelScale(modelId, scale, scale, scale);
            game.SetModelStatic(modelId, true);
            game.AddModelToScene(sceneId, modelId);

            placed++;
        }

        return placed;
    }

    static void PrintControls(GameConfig config)
    {
        Console.WriteLine("========================================");
        Console.WriteLine("  CHARACTER SANDBOX - GoudEngine");
        Console.WriteLine("========================================");
        Console.WriteLine("  W/A/S/D    Move (camera-relative)");
        Console.WriteLine("  Shift      Hold to run");
        Console.WriteLine("  Arrows     Camera yaw & pitch");
        Console.WriteLine("  G          Toggle grid");
        Console.WriteLine("  F          Toggle fog");
        Console.WriteLine("  +/-        Spawn/remove 10 agents");
        Console.WriteLine("  1-5        Perf toggles (culling/skin/mat/lod/eval)");
        Console.WriteLine("  6          Toggle animation phase-lock");
        Console.WriteLine("  ESC        Quit");
        Console.WriteLine("========================================");
        Console.WriteLine($"  NPCs: {config.NpcCount}  Animals: {config.AnimalCount}  Seed: {config.Seed}");
        Console.WriteLine($"  VariedAnims: {config.VariedAnims}  PhaseLock: {config.PhaseLock}");
        Console.WriteLine("  CLI: --npcs N --animals N --seed N --varied-anims --phase-lock");
        Console.WriteLine();
    }
}
