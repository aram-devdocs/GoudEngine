// GameManager.cs
// Main game manager using immediate-mode rendering

using GoudEngine.Input;
using GoudEngine.Math;
using IsometricRpg.Core;
using IsometricRpg.Player;
using IsometricRpg.Enemies;
using IsometricRpg.NPCs;
using IsometricRpg.Combat;
using IsometricRpg.UI;

namespace IsometricRpg;

public class GameManager
{
    private readonly GoudGame _game;
    private GameStateManager _stateManager = null!;

    // Screen dimensions
    public const uint ScreenWidth = 800;
    public const uint ScreenHeight = 600;

    // Game entities
    private SimplePlayer _player = null!;
    private readonly List<SimpleEnemy> _enemies = new();
    private readonly List<SimpleNPC> _npcs = new();
    private readonly List<SimpleProjectile> _projectiles = new();

    // Textures
    private ulong _playerTexture;
    private ulong _enemyTexture;
    private ulong _npcTexture;
    private ulong _projectileTexture;
    private ulong _groundTexture;
    private ulong _titleTexture;
    private ulong _pressStartTexture;
    private ulong _healthBgTexture;
    private ulong _healthFillTexture;
    private ulong _dialogueBoxTexture;

    // NPC interaction
    private SimpleNPC? _nearbyNpc;
    private float _interactionCooldown;

    // Enemy spawning
    private float _enemySpawnTimer;
    private const float EnemySpawnInterval = 5f;
    private const int MaxEnemies = 5;

    // Input cooldown
    private float _startCooldown;

    // UI
    private SimpleUI _ui = null!;

    public GameManager(GoudGame game)
    {
        _game = game;
    }

    public void Initialize()
    {
        _stateManager = new GameStateManager();
        _stateManager.OnStateChanged += HandleStateChanged;

        // Load textures
        LoadTextures();

        // Create player
        _player = new SimplePlayer(400, 300);

        // Create NPC
        CreateNPC();

        // Create UI
        _ui = new SimpleUI(_game, _titleTexture, _pressStartTexture, 
            _healthBgTexture, _healthFillTexture, _dialogueBoxTexture);

        _game.GameLog("Isometric RPG initialized");
    }

    private void LoadTextures()
    {
        _playerTexture = _game.LoadTexture("assets/sprites/player/player.png");
        _enemyTexture = _game.LoadTexture("assets/sprites/enemies/enemy.png");
        _npcTexture = _game.LoadTexture("assets/sprites/npcs/npc.png");
        _projectileTexture = _game.LoadTexture("assets/sprites/projectiles/projectile.png");
        _groundTexture = _game.LoadTexture("assets/sprites/tiles/ground_grid.png");
        _titleTexture = _game.LoadTexture("assets/sprites/ui/title.png");
        _pressStartTexture = _game.LoadTexture("assets/sprites/ui/press_start.png");
        _healthBgTexture = _game.LoadTexture("assets/sprites/ui/health_bg.png");
        _healthFillTexture = _game.LoadTexture("assets/sprites/ui/health_fill.png");
        _dialogueBoxTexture = _game.LoadTexture("assets/sprites/ui/dialogue_box.png");
    }

    private void CreateNPC()
    {
        var npc = new SimpleNPC(600, 300, "Village Elder");
        _npcs.Add(npc);
    }

    private void SpawnEnemy()
    {
        if (_enemies.Count >= MaxEnemies) return;

        // Spawn at random edge
        float x, y;
        int edge = Random.Shared.Next(4);
        switch (edge)
        {
            case 0: x = Random.Shared.Next(50, (int)ScreenWidth - 50); y = 50; break;
            case 1: x = ScreenWidth - 50; y = Random.Shared.Next(50, (int)ScreenHeight - 50); break;
            case 2: x = Random.Shared.Next(50, (int)ScreenWidth - 50); y = ScreenHeight - 50; break;
            default: x = 50; y = Random.Shared.Next(50, (int)ScreenHeight - 50); break;
        }

        var enemy = new SimpleEnemy(x, y);
        _enemies.Add(enemy);
        _game.GameLog($"Enemy spawned at ({x:F0}, {y:F0})");
    }

    public void Start()
    {
        _stateManager.SetState(GameState.Title);
        _startCooldown = 0.5f;
        _game.GameLog("Game started - showing title screen");
    }

    public void Update(float deltaTime)
    {
        // Update cooldowns
        if (_startCooldown > 0) _startCooldown -= deltaTime;
        if (_interactionCooldown > 0) _interactionCooldown -= deltaTime;

        HandleGlobalInput();

        switch (_stateManager.CurrentState)
        {
            case GameState.Title:
                UpdateTitleScreen(deltaTime);
                break;
            case GameState.Playing:
                UpdatePlaying(deltaTime);
                break;
            case GameState.Dialogue:
                UpdateDialogue(deltaTime);
                break;
            case GameState.GameOver:
                UpdateGameOver(deltaTime);
                break;
        }
    }

    public void Draw()
    {
        switch (_stateManager.CurrentState)
        {
            case GameState.Title:
                DrawTitleScreen();
                break;
            case GameState.Playing:
                DrawPlaying();
                break;
            case GameState.Dialogue:
                DrawPlaying();
                DrawDialogue();
                break;
            case GameState.GameOver:
                DrawPlaying();
                DrawGameOver();
                break;
        }
    }

    private void HandleGlobalInput()
    {
        if (_game.IsKeyPressed(Keys.Escape))
        {
            if (_stateManager.CurrentState == GameState.Playing)
            {
                _stateManager.SetState(GameState.Title);
            }
            else if (_stateManager.CurrentState == GameState.Dialogue)
            {
                _stateManager.SetState(GameState.Playing);
            }
        }
    }

    private void UpdateTitleScreen(float deltaTime)
    {
        if (_game.IsKeyPressed(Keys.Space) && _startCooldown <= 0)
        {
            StartGame();
        }
    }

    private void StartGame()
    {
        _player.Reset(400, 300);
        _enemies.Clear();
        _projectiles.Clear();
        SpawnEnemy();
        _enemySpawnTimer = EnemySpawnInterval;
        _stateManager.SetState(GameState.Playing);
        _game.GameLog("=== GAME STARTED ===");
    }

    private void UpdatePlaying(float deltaTime)
    {
        // Update player
        UpdatePlayerInput(deltaTime);
        _player.Update(deltaTime);

        // Update enemies
        for (int i = _enemies.Count - 1; i >= 0; i--)
        {
            var enemy = _enemies[i];
            enemy.MoveTowards(_player.X, _player.Y, deltaTime);
            enemy.Update(deltaTime);

            // Check collision with player
            if (enemy.CollidesWith(_player) && !enemy.IsDead)
            {
                _player.TakeDamage(10);
                enemy.TakeDamage(100); // Kill on contact for simplicity
            }

            // Remove dead enemies
            if (enemy.IsDead && enemy.DeathTimer <= 0)
            {
                _enemies.RemoveAt(i);
            }
        }

        // Update projectiles
        for (int i = _projectiles.Count - 1; i >= 0; i--)
        {
            var proj = _projectiles[i];
            proj.Update(deltaTime);

            // Check collision with enemies
            foreach (var enemy in _enemies)
            {
                if (!enemy.IsDead && proj.CollidesWith(enemy))
                {
                    enemy.TakeDamage(25);
                    proj.IsActive = false;
                    break;
                }
            }

            // Remove inactive projectiles
            if (!proj.IsActive)
            {
                _projectiles.RemoveAt(i);
            }
        }

        // Check NPC interaction
        CheckNPCInteraction();

        // Enemy spawning
        _enemySpawnTimer -= deltaTime;
        if (_enemySpawnTimer <= 0)
        {
            SpawnEnemy();
            _enemySpawnTimer = EnemySpawnInterval;
        }

        // Check player death
        if (_player.IsDead)
        {
            _stateManager.SetState(GameState.GameOver);
            _startCooldown = 0.5f;
            _game.GameLog("=== GAME OVER ===");
        }
    }

    private void UpdatePlayerInput(float deltaTime)
    {
        float speed = 200f;
        float dx = 0, dy = 0;

        if (_game.IsKeyPressed(Keys.W)) dy -= speed * deltaTime;
        if (_game.IsKeyPressed(Keys.S)) dy += speed * deltaTime;
        if (_game.IsKeyPressed(Keys.A)) dx -= speed * deltaTime;
        if (_game.IsKeyPressed(Keys.D)) dx += speed * deltaTime;

        _player.Move(dx, dy);

        // Shooting with left mouse button
        if (_game.IsMouseButtonJustPressed(MouseButtons.Left))
        {
            var (mx, my) = _game.GetMousePosition();
            ShootProjectile(_player.X, _player.Y, mx, my);
        }
    }

    private void ShootProjectile(float fromX, float fromY, float toX, float toY)
    {
        var proj = new SimpleProjectile(fromX, fromY, toX, toY);
        _projectiles.Add(proj);
    }

    private void CheckNPCInteraction()
    {
        _nearbyNpc = null;

        foreach (var npc in _npcs)
        {
            if (npc.IsNearPlayer(_player.X, _player.Y))
            {
                _nearbyNpc = npc;
                break;
            }
        }

        if (_nearbyNpc != null && _game.IsKeyPressed(Keys.E) && _interactionCooldown <= 0)
        {
            _stateManager.SetState(GameState.Dialogue);
            _interactionCooldown = 0.5f;
        }
    }

    private void UpdateDialogue(float deltaTime)
    {
        if (_game.IsKeyPressed(Keys.Space) && _startCooldown <= 0)
        {
            _stateManager.SetState(GameState.Playing);
            _startCooldown = 0.3f;
        }
    }

    private void UpdateGameOver(float deltaTime)
    {
        if (_game.IsKeyPressed(Keys.R))
        {
            StartGame();
        }
        if (_game.IsKeyPressed(Keys.Space) && _startCooldown <= 0)
        {
            _stateManager.SetState(GameState.Title);
            _startCooldown = 0.5f;
        }
    }

    // Drawing methods
    private void DrawTitleScreen()
    {
        // Draw title background
        _game.DrawQuad(ScreenWidth / 2, ScreenHeight / 2, ScreenWidth, ScreenHeight,
            new Color(0.1f, 0.15f, 0.1f, 1f));

        // Draw title
        _game.DrawSprite(_titleTexture, ScreenWidth / 2, 200, 400, 100);

        // Draw "Press Start" text (blinking)
        float blink = (float)System.Math.Sin(Environment.TickCount / 300.0) * 0.5f + 0.5f;
        if (blink > 0.3f)
        {
            _game.DrawSprite(_pressStartTexture, ScreenWidth / 2, 400, 200, 50);
        }
    }

    private void DrawPlaying()
    {
        // Draw ground/grid
        _game.DrawSprite(_groundTexture, ScreenWidth / 2, ScreenHeight / 2, ScreenWidth, ScreenHeight);

        // Draw NPCs
        foreach (var npc in _npcs)
        {
            _game.DrawSprite(_npcTexture, npc.X + 16, npc.Y + 16, 32, 32);
            
            // Draw interaction hint if near
            if (npc.IsNearPlayer(_player.X, _player.Y))
            {
                _game.DrawQuad(npc.X + 16, npc.Y - 10, 40, 16, new Color(0.2f, 0.2f, 0.2f, 0.8f));
            }
        }

        // Draw enemies
        foreach (var enemy in _enemies)
        {
            if (!enemy.IsDead)
            {
                float flash = enemy.HitFlash > 0 ? 1f : 0f;
                _game.DrawSprite(_enemyTexture, enemy.X + 16, enemy.Y + 16, 32, 32);
            }
        }

        // Draw projectiles
        foreach (var proj in _projectiles)
        {
            _game.DrawSprite(_projectileTexture, proj.X + 8, proj.Y + 8, 16, 16);
        }

        // Draw player
        float playerFlash = _player.HitFlash > 0 ? 0.5f : 0f;
        _game.DrawSprite(_playerTexture, _player.X + 16, _player.Y + 16, 32, 32);

        // Draw health bar
        DrawHealthBar();
    }

    private void DrawHealthBar()
    {
        float barX = 100;
        float barY = 30;
        float barWidth = 150;
        float barHeight = 20;

        // Background
        _game.DrawSprite(_healthBgTexture, barX, barY, barWidth, barHeight);

        // Fill based on health
        float healthPercent = (float)_player.Health / _player.MaxHealth;
        float fillWidth = barWidth * healthPercent;
        _game.DrawSprite(_healthFillTexture, barX - (barWidth - fillWidth) / 2, barY, fillWidth, barHeight - 4);
    }

    private void DrawDialogue()
    {
        // Draw dialogue box
        float boxX = ScreenWidth / 2;
        float boxY = ScreenHeight - 100;
        _game.DrawSprite(_dialogueBoxTexture, boxX, boxY, 600, 150);

        // Would draw text here if we had text rendering
    }

    private void DrawGameOver()
    {
        // Darken screen
        _game.DrawQuad(ScreenWidth / 2, ScreenHeight / 2, ScreenWidth, ScreenHeight,
            new Color(0f, 0f, 0f, 0.7f));

        // Game over text (using title as placeholder)
        _game.DrawQuad(ScreenWidth / 2, ScreenHeight / 2 - 50, 300, 60, new Color(0.8f, 0.2f, 0.2f, 1f));
        _game.DrawQuad(ScreenWidth / 2, ScreenHeight / 2 + 30, 250, 30, new Color(0.3f, 0.3f, 0.3f, 1f));
    }

    private void HandleStateChanged(GameState oldState, GameState newState)
    {
        _game.GameLog($"State: {oldState} -> {newState}");
    }
}
