using System;
using GoudEngine;

static class SceneSetup
{
    static bool _gridEnabled = false;
    static bool _fogEnabled = false;
    static bool _frustumCulling = true;
    static bool _gpuSkinning = true;
    static bool _materialSorting = true;
    static bool _animationLod = true;
    static bool _sharedAnimEval = true;

    public static uint Initialize(GoudGame game)
    {
        uint sceneId = game.CreateScene("main");
        game.SetCurrentScene(sceneId);

        game.ConfigureSkybox(enabled: true, r: 0.78f, g: 0.86f, b: 0.96f, a: 1.0f);
        game.ConfigureFog(enabled: false, r: 0.78f, g: 0.86f, b: 0.96f, density: 0.0035f);
        game.ConfigureGrid(enabled: false, size: 200.0f, divisions: 100);

        AddThreePointLighting(game, sceneId);
        AddGroundPlane(game, sceneId);

        game.SetFrustumCullingEnabled(true);
        game.SetSkinningMode(1u);
        game.SetMaterialSortingEnabled(true);
        game.SetAnimationLodEnabled(true);
        game.SetSharedAnimationEval(true);

        return sceneId;
    }

    public static void HandleToggles(GoudGame game)
    {
        bool anyMod = game.IsKeyPressed(Keys.LeftShift) || game.IsKeyPressed(Keys.RightShift)
            || game.IsKeyPressed(Keys.LeftControl) || game.IsKeyPressed(Keys.RightControl)
            || game.IsKeyPressed(Keys.LeftAlt) || game.IsKeyPressed(Keys.RightAlt)
            || game.IsKeyPressed(Keys.Tab);

        if (game.IsKeyJustPressed(Keys.G))
        {
            _gridEnabled = !_gridEnabled;
            game.SetGridEnabled(_gridEnabled);
            Console.WriteLine($"Grid {(_gridEnabled ? "ON" : "OFF")}");
        }

        if (game.IsKeyJustPressed(Keys.F))
        {
            _fogEnabled = !_fogEnabled;
            game.SetFogEnabled(_fogEnabled);
            Console.WriteLine($"Fog {(_fogEnabled ? "ON" : "OFF")}");
        }

        if (anyMod) return;

        if (game.IsKeyJustPressed(Keys.Digit1))
        {
            _frustumCulling = !_frustumCulling;
            game.SetFrustumCullingEnabled(_frustumCulling);
            Console.WriteLine($"Frustum culling {(_frustumCulling ? "ON" : "OFF")}");
        }

        if (game.IsKeyJustPressed(Keys.Digit2))
        {
            // Skinning mode cannot be toggled at runtime — models are loaded
            // with GPU-specific vertex format and cannot switch to CPU mode
            // without reloading.
            Console.WriteLine($"Skinning: GPU (fixed at load time)");
        }

        if (game.IsKeyJustPressed(Keys.Digit3))
        {
            _materialSorting = !_materialSorting;
            game.SetMaterialSortingEnabled(_materialSorting);
            Console.WriteLine($"Material sorting {(_materialSorting ? "ON" : "OFF")}");
        }

        if (game.IsKeyJustPressed(Keys.Digit4))
        {
            _animationLod = !_animationLod;
            game.SetAnimationLodEnabled(_animationLod);
            Console.WriteLine($"Animation LOD {(_animationLod ? "ON" : "OFF")}");
        }

        if (game.IsKeyJustPressed(Keys.Digit5))
        {
            _sharedAnimEval = !_sharedAnimEval;
            game.SetSharedAnimationEval(_sharedAnimEval);
            Console.WriteLine($"Shared anim eval {(_sharedAnimEval ? "ON" : "OFF")}");
        }
    }

    static void AddThreePointLighting(GoudGame game, uint sceneId)
    {
        // Sun: warm directional from above
        uint sun = game.AddLight(1, 0f, 25f, 0f, -0.15f, -1f, 0.2f, 1.0f, 0.98f, 0.93f, 1.9f, 120f, 0f);
        game.AddLightToScene(sceneId, sun);

        // Fill: cool blue from the side
        uint fill = game.AddLight(1, -20f, 15f, 20f, 0.35f, -0.85f, -0.25f, 0.72f, 0.78f, 0.88f, 0.95f, 120f, 0f);
        game.AddLightToScene(sceneId, fill);

        // Rim: warm amber backlight
        uint rim = game.AddLight(1, 24f, 18f, -18f, -0.45f, -0.8f, 0.15f, 0.92f, 0.86f, 0.74f, 0.65f, 120f, 0f);
        game.AddLightToScene(sceneId, rim);
    }

    static void AddGroundPlane(GoudGame game, uint sceneId)
    {
        uint plane = game.CreatePlane(0, 200f, 200f);
        game.SetObjectPosition(plane, 0f, 0f, 0f);

        uint mat = game.CreateMaterial(0, 0.50f, 0.68f, 0.40f, 1f, 16f, 0f, 0.8f, 0.1f);
        game.SetObjectMaterial(plane, mat);
        game.AddObjectToScene(sceneId, plane);
    }
}
