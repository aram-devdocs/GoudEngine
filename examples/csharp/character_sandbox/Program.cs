using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
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
            .SetVsync(config.Vsync)
            .Build();

        uint sceneId = SceneSetup.Initialize(game, config.Shadows, config.ShadowSize);
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
        {
            string assetsDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "assets");
            throw new InvalidOperationException($"No humanoid model found. Check {assetsDir}/Character.glb exists.");
        }

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

        if (config.Profile)
        {
            RunProfile(game, crowd, config);
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
                var pt = game.GetFramePhaseTimings();

                Console.Write(
                    $"\rFPS: {fpsFrames / fpsTimer:F1}  Agents: {crowd.Count}  " +
                    $"Draws: {draws}  Visible: {visible}/{visible + culled}  " +
                    $"Instanced: {instanced}  Instances: {activeInst}  " +
                    $"AnimEval: {animEval}  Saved: {animSaved}  Bones: {boneUploads}  " +
                    $"[anim={pt.AnimEvalUs}us bone={pt.BonePackUs}+{pt.BoneUploadUs}us " +
                    $"shadow={pt.ShadowPassUs}us render={pt.RenderPassUs}us " +
                    $"present={pt.SurfacePresentUs}us]   "
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

    static void RunProfile(GoudGame game, CrowdSystem crowd, GameConfig config)
    {
        // Fixed camera for reproducible results
        game.SetCameraPosition3D(0f, 8f, 15f);
        game.SetCameraRotation3D(-25f, 0f, 0f);

        // Fixed timestep at 30Hz matching Throne's sim rate
        game.SetFixedTimestep(1f / 30f);
        game.SetMaxFixedSteps(8);

        float durationSec = config.ProfileDuration;
        float elapsed = 0f;
        int frameCount = 0;
        var samples = new List<ProfileSample>();
        var sw = Stopwatch.StartNew();

        Console.WriteLine($"PROFILE: Starting {durationSec}s profile with {crowd.Count} agents " +
            $"(shadows={config.Shadows}, shadow_size={config.ShadowSize}, vsync={config.Vsync})");

        while (elapsed < durationSec && !game.ShouldClose())
        {
            var frameStart = sw.Elapsed;

            game.BeginFrame(0.1f, 0.1f, 0.15f, 1.0f);
            float dt = game.DeltaTime;

            var simStart = sw.Elapsed;
            crowd.Update(dt);
            game.UpdateAnimations(dt);
            float simMs = (float)(sw.Elapsed - simStart).TotalMilliseconds;

            var renderStart = sw.Elapsed;
            game.Render3D();
            game.EndFrame();
            float renderMs = (float)(sw.Elapsed - renderStart).TotalMilliseconds;

            float frameMs = (float)(sw.Elapsed - frameStart).TotalMilliseconds;
            elapsed += dt;
            frameCount++;

            samples.Add(new ProfileSample
            {
                FrameIndex = frameCount,
                DeltaTimeMs = dt * 1000f,
                FrameTimeMs = frameMs,
                SimTimeMs = simMs,
                RenderTimeMs = renderMs,
                OverheadMs = frameMs - simMs - renderMs,
                EntityCount = crowd.Count,
            });
        }

        sw.Stop();

        // Compute statistics
        float avgFps = frameCount / Math.Max(elapsed, 0.001f);
        float avgFrame = 0f, minFrame = float.MaxValue, maxFrame = 0f;
        float avgSim = 0f, avgRender = 0f, avgOverhead = 0f;
        foreach (var s in samples)
        {
            avgFrame += s.FrameTimeMs;
            avgSim += s.SimTimeMs;
            avgRender += s.RenderTimeMs;
            avgOverhead += s.OverheadMs;
            if (s.FrameTimeMs < minFrame) minFrame = s.FrameTimeMs;
            if (s.FrameTimeMs > maxFrame) maxFrame = s.FrameTimeMs;
        }
        int n = Math.Max(samples.Count, 1);
        avgFrame /= n;
        avgSim /= n;
        avgRender /= n;
        avgOverhead /= n;

        int draws = game.GetDrawCalls();
        int visible = game.GetVisibleObjectCount();
        int culled = game.GetCulledObjectCount();
        int instanced = game.GetInstancedDrawCalls();
        int animEval = game.GetAnimationEvaluationCount();
        int animSaved = game.GetAnimationEvaluationSavedCount();
        var pt = game.GetFramePhaseTimings();

        // Console summary
        Console.WriteLine();
        Console.WriteLine($"PROFILE RESULTS ({frameCount} frames, {elapsed:F1}s):");
        Console.WriteLine($"  Avg FPS: {avgFps:F1}");
        Console.WriteLine($"  Avg Sim Time: {avgSim:F1}ms");
        Console.WriteLine($"  Avg Render Time: {avgRender:F1}ms");
        Console.WriteLine($"  Avg Overhead: {avgOverhead:F1}ms");
        Console.WriteLine($"  Frame Time: Min {minFrame:F2}ms / Max {maxFrame:F2}ms / Avg {avgFrame:F2}ms");
        Console.WriteLine($"  Draws: {draws}  Visible: {visible}/{visible + culled}  Instanced: {instanced}");
        Console.WriteLine($"  AnimEval: {animEval}  Saved: {animSaved}  Agents: {crowd.Count}");
        Console.WriteLine($"  Phase Timings (last frame, us):");
        Console.WriteLine($"    surface_acquire={pt.SurfaceAcquireUs}  shadow_pass={pt.ShadowPassUs}  shadow_build={pt.ShadowBuildUs}");
        Console.WriteLine($"    render3d_scene={pt.Render3dSceneUs}  uniform_upload={pt.UniformUploadUs}  render_pass={pt.RenderPassUs}");
        Console.WriteLine($"    gpu_submit={pt.GpuSubmitUs}  readback_stall={pt.ReadbackStallUs}  surface_present={pt.SurfacePresentUs}");
        Console.WriteLine($"    anim_eval={pt.AnimEvalUs}  bone_pack={pt.BonePackUs}  bone_upload={pt.BoneUploadUs}");

        // Write markdown report
        string profilingDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "profiling");
        Directory.CreateDirectory(profilingDir);
        string timestamp = DateTime.Now.ToString("yyyyMMdd-HHmmss");
        string mdPath = Path.Combine(profilingDir, $"profile-{timestamp}.md");

        var md = new System.Text.StringBuilder();
        md.AppendLine($"# Character Sandbox Profile — {timestamp}");
        md.AppendLine();
        md.AppendLine("## Configuration");
        md.AppendLine($"- NPCs: {config.NpcCount}, Animals: {config.AnimalCount}");
        md.AppendLine($"- Shadows: {config.Shadows} (size: {config.ShadowSize})");
        md.AppendLine($"- VSync: {config.Vsync}");
        md.AppendLine($"- Duration: {durationSec}s");
        md.AppendLine($"- Frames: {frameCount}");
        md.AppendLine();
        md.AppendLine("## Results");
        md.AppendLine($"- **Avg FPS: {avgFps:F1}**");
        md.AppendLine($"- Avg Sim: {avgSim:F1}ms");
        md.AppendLine($"- Avg Render: {avgRender:F1}ms");
        md.AppendLine($"- Avg Overhead: {avgOverhead:F1}ms");
        md.AppendLine($"- Frame Time: Min {minFrame:F2}ms / Max {maxFrame:F2}ms / Avg {avgFrame:F2}ms");
        md.AppendLine();
        md.AppendLine("## Renderer Stats");
        md.AppendLine($"- Draw calls: {draws}");
        md.AppendLine($"- Visible: {visible}/{visible + culled}");
        md.AppendLine($"- Instanced: {instanced}");
        md.AppendLine($"- AnimEval: {animEval}, Saved: {animSaved}");
        md.AppendLine();
        md.AppendLine("## Phase Timings (last frame, microseconds)");
        md.AppendLine($"| Phase | Time (us) |");
        md.AppendLine($"|-------|-----------|");
        md.AppendLine($"| surface_acquire | {pt.SurfaceAcquireUs} |");
        md.AppendLine($"| shadow_pass | {pt.ShadowPassUs} |");
        md.AppendLine($"| shadow_build | {pt.ShadowBuildUs} |");
        md.AppendLine($"| render3d_scene | {pt.Render3dSceneUs} |");
        md.AppendLine($"| uniform_upload | {pt.UniformUploadUs} |");
        md.AppendLine($"| render_pass | {pt.RenderPassUs} |");
        md.AppendLine($"| gpu_submit | {pt.GpuSubmitUs} |");
        md.AppendLine($"| readback_stall | {pt.ReadbackStallUs} |");
        md.AppendLine($"| surface_present | {pt.SurfacePresentUs} |");
        md.AppendLine($"| anim_eval | {pt.AnimEvalUs} |");
        md.AppendLine($"| bone_pack | {pt.BonePackUs} |");
        md.AppendLine($"| bone_upload | {pt.BoneUploadUs} |");

        File.WriteAllText(mdPath, md.ToString());
        Console.WriteLine($"\nReport written to: {mdPath}");

        // Write JSON for automated comparison
        string jsonPath = Path.Combine(profilingDir, $"profile-{timestamp}.json");
        var json = new System.Text.StringBuilder();
        json.AppendLine("{");
        json.AppendLine($"  \"timestamp\": \"{timestamp}\",");
        json.AppendLine($"  \"npcs\": {config.NpcCount},");
        json.AppendLine($"  \"animals\": {config.AnimalCount},");
        json.AppendLine($"  \"shadows\": {config.Shadows.ToString().ToLower()},");
        json.AppendLine($"  \"shadow_size\": {config.ShadowSize},");
        json.AppendLine($"  \"vsync\": {config.Vsync.ToString().ToLower()},");
        json.AppendLine($"  \"duration_s\": {durationSec},");
        json.AppendLine($"  \"frames\": {frameCount},");
        json.AppendLine($"  \"avg_fps\": {avgFps:F1},");
        json.AppendLine($"  \"avg_sim_ms\": {avgSim:F1},");
        json.AppendLine($"  \"avg_render_ms\": {avgRender:F1},");
        json.AppendLine($"  \"avg_overhead_ms\": {avgOverhead:F1},");
        json.AppendLine($"  \"min_frame_ms\": {minFrame:F2},");
        json.AppendLine($"  \"max_frame_ms\": {maxFrame:F2},");
        json.AppendLine($"  \"avg_frame_ms\": {avgFrame:F2},");
        json.AppendLine($"  \"draw_calls\": {draws},");
        json.AppendLine($"  \"visible\": {visible},");
        json.AppendLine($"  \"culled\": {culled},");
        json.AppendLine($"  \"instanced\": {instanced},");
        json.AppendLine($"  \"anim_eval\": {animEval},");
        json.AppendLine($"  \"anim_saved\": {animSaved},");
        json.AppendLine($"  \"agents\": {crowd.Count}");
        json.AppendLine("}");
        File.WriteAllText(jsonPath, json.ToString());

        // Write stats to temp file for automated comparison (backward compat)
        File.WriteAllText(Path.Combine(Path.GetTempPath(), "goud_profile_stats.txt"),
            $"fps={avgFps:F1}\ndraws={draws}\ninstanced={instanced}\n" +
            $"agents={crowd.Count}\nanim_eval={animEval}\nanim_saved={animSaved}\n");
    }

    struct ProfileSample
    {
        public int FrameIndex;
        public float DeltaTimeMs;
        public float FrameTimeMs;
        public float SimTimeMs;
        public float RenderTimeMs;
        public float OverheadMs;
        public int EntityCount;
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
        Console.WriteLine("  1,3-5      Perf toggles (culling/mat/lod/eval)");
        Console.WriteLine("  6          Toggle animation phase-lock");
        Console.WriteLine("  ESC        Quit");
        Console.WriteLine("========================================");
        Console.WriteLine($"  NPCs: {config.NpcCount}  Animals: {config.AnimalCount}  Seed: {config.Seed}");
        Console.WriteLine($"  VariedAnims: {config.VariedAnims}  PhaseLock: {config.PhaseLock}");
        Console.WriteLine($"  Shadows: {config.Shadows}  ShadowSize: {config.ShadowSize}  VSync: {config.Vsync}");
        Console.WriteLine("  CLI: --npcs N --animals N --seed N --varied-anims --phase-lock");
        Console.WriteLine("       --profile --duration N --shadows --shadow-size N --no-vsync");
        Console.WriteLine();
    }
}
