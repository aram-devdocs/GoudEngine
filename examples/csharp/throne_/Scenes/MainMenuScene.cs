// Scenes/MainMenuScene.cs
// Main menu scene with 3D bouncing cube and 2D UI overlay

using GoudEngine.Input;
using GoudEngine.Math;
using Throne.UI;

namespace Throne.Scenes;

/// <summary>
/// Main menu scene featuring a 3D bouncing cube with 2D UI overlay.
/// Demonstrates mixed 2D/3D rendering pattern.
/// </summary>
public class MainMenuScene : IScene
{
    private readonly GoudGame _game;
    private readonly uint _screenWidth;
    private readonly uint _screenHeight;

    // 3D cube
    private uint _cubeId;
    private float _cubeTime;
    private float _cubeRotation;
    private const float BounceHeight = 1.0f;
    private const float BounceSpeed = 2.5f;
    private const float BaseHeight = 2.0f;
    private const float RotationSpeed = 30f;

    // Lighting
    private uint _mainLight;
    private uint _accentLight;

    // Camera
    private const float CameraX = 0f;
    private const float CameraY = 4f;
    private const float CameraZ = -8f;
    private const float CameraPitch = -20f;

    // UI animation
    private float _uiAnimTime;

    public string Name => "MainMenu";

    public MainMenuScene(GoudGame game, uint screenWidth, uint screenHeight)
    {
        _game = game;
        _screenWidth = screenWidth;
        _screenHeight = screenHeight;
    }

    public void Enter()
    {
        _game.GameLog("Entering Main Menu Scene");

        // Create 3D cube (no texture, will use default material)
        _cubeId = _game.CreateCube(0, 1.5f, 1.5f, 1.5f);
        _game.SetObjectPosition(_cubeId, 0f, BaseHeight, 0f);

        // Configure 3D environment - NO GRID for clean menu look
        _game.ConfigureGrid(enabled: false, size: 15f, divisions: 15);
        _game.ConfigureSkybox(enabled: true, r: 0.02f, g: 0.02f, b: 0.05f, a: 1.0f);
        _game.ConfigureFog(enabled: false, r: 0.02f, g: 0.02f, b: 0.05f, density: 0.03f);

        // Add lighting
        _mainLight = _game.AddPointLight(3f, 6f, -3f, 1.0f, 0.95f, 0.8f, 3.0f, 20f);
        _accentLight = _game.AddPointLight(-3f, 4f, 3f, 0.3f, 0.5f, 1.0f, 2.0f, 15f);

        // Set camera
        _game.SetCameraPosition3D(CameraX, CameraY, CameraZ);
        _game.SetCameraRotation(CameraPitch, 0f, 0f);

        _cubeTime = 0f;
        _cubeRotation = 0f;
        _uiAnimTime = 0f;
    }

    public void Exit()
    {
        _game.GameLog("Exiting Main Menu Scene");

        // Cleanup 3D objects
        if (_cubeId != GoudGame.InvalidObject)
        {
            _game.DestroyObject(_cubeId);
        }
        _game.RemoveLight(_mainLight);
        _game.RemoveLight(_accentLight);
    }

    public void Update(float deltaTime)
    {
        // Update cube animation
        _cubeTime += deltaTime;
        _cubeRotation += RotationSpeed * deltaTime;
        _uiAnimTime += deltaTime;

        // Bounce animation
        float bounceOffset = (float)Math.Sin(_cubeTime * BounceSpeed) * BounceHeight;
        _game.SetObjectPosition(_cubeId, 0f, BaseHeight + bounceOffset, 0f);
        _game.SetObjectRotation(_cubeId, _cubeRotation * 0.3f, _cubeRotation, _cubeRotation * 0.5f);

        // Handle escape key
        if (_game.IsKeyJustPressed(Keys.Escape))
        {
            _game.Close();
        }
    }

    public void Draw()
    {
        // First: Render 3D scene
        _game.Render3D();

        // Then: Render 2D UI overlay on top
        DrawUI();
    }

    private void DrawUI()
    {
        // Title bar at top - gold colored
        float titleY = 60f;
        float titleFloat = (float)Math.Sin(_uiAnimTime * 2f) * 5f;
        _game.DrawQuad(_screenWidth / 2f, titleY + titleFloat, 
            280f, 50f, new Color(0.7f, 0.55f, 0.2f, 1f));
        
        // Inner title highlight
        _game.DrawQuad(_screenWidth / 2f, titleY + titleFloat, 
            270f, 40f, new Color(0.85f, 0.7f, 0.3f, 1f));

        // Bottom UI panel
        _game.DrawQuad(_screenWidth / 2f, _screenHeight - 80f, 
            _screenWidth, 120f, new Color(0f, 0f, 0f, 0.7f));

        // New Game button - grayed out (locked)
        float buttonY = _screenHeight - 80f;
        _game.DrawQuad(_screenWidth / 2f, buttonY, 
            180f, 40f, new Color(0.3f, 0.3f, 0.35f, 0.9f));
        
        // Button inner
        _game.DrawQuad(_screenWidth / 2f, buttonY, 
            170f, 32f, new Color(0.4f, 0.4f, 0.45f, 0.9f));

        // Lock indicator (small square)
        _game.DrawQuad(_screenWidth / 2f + 70f, buttonY, 
            20f, 20f, new Color(0.6f, 0.5f, 0.4f, 1f));
    }
}

