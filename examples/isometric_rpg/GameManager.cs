using GoudEngine;
using GoudEngine.Input;
using CsBindgen;
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
    public const int ScreenWidth = 800;
    public const int ScreenHeight = 600;

    // Game entities
    private Player.Player _player = null!;
    private readonly List<Enemy> _enemies = new();
    private readonly List<NPC> _npcs = new();

    // Systems
    private CombatSystem _combatSystem = null!;
    private DialogueSystem _dialogueSystem = null!;
    private UIManager _uiManager = null!;

    // Textures
    private uint _playerTexture;
    private uint _enemyTexture;
    private uint _npcTexture;
    private uint _projectileTexture;
    private uint _groundTexture;
    private uint _healthBgTexture;
    private uint _healthFillTexture;
    private uint _dialogueBoxTexture;
    private uint _arrowTexture;
    private uint _titleTexture;
    private uint _pressStartTexture;

    // Ground sprite
    private uint _groundSpriteId;

    // NPC interaction
    private NPC? _nearbyNpc;
    private float _interactionCooldown;

    // Enemy spawning
    private float _enemySpawnTimer;
    private const float EnemySpawnInterval = 5f;
    private const int MaxEnemies = 5;

    // Input cooldown
    private float _startCooldown;

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

        // Create ground sprite first (lowest z-layer)
        _groundSpriteId = _game.AddSprite(new SpriteCreateDto
        {
            texture_id = _groundTexture,
            x = 0,
            y = 0,
            z_layer = 0
        });

        // Initialize systems
        _combatSystem = new CombatSystem(_game);
        _combatSystem.Initialize(_projectileTexture);
        _combatSystem.OnEntityKilled += OnEntityKilled;

        _dialogueSystem = new DialogueSystem(_game);
        _dialogueSystem.OnDialogueEnded += OnDialogueEnded;

        _uiManager = new UIManager(_game);
        _uiManager.Initialize(
            _dialogueSystem,
            _healthBgTexture,
            _healthFillTexture,
            _dialogueBoxTexture,
            _arrowTexture,
            _titleTexture,
            _pressStartTexture
        );

        // Create player
        _player = new Player.Player(_game);
        _player.Initialize();
        _player.SetupSprite(_playerTexture);
        _combatSystem.RegisterPlayer(_player);

        // Subscribe to player death
        _player.Health.OnDeath += OnPlayerDeath;
        _player.Health.OnHealthChanged += OnPlayerHealthChanged;

        // Create NPC (position near center-right)
        CreateNPC();

        _game.GameLog("Isometric RPG initialized");
    }

    private void LoadTextures()
    {
        // Load character textures (new isometric style)
        _playerTexture = _game.CreateTexture("assets/sprites/player/player.png");
        _enemyTexture = _game.CreateTexture("assets/sprites/enemies/enemy.png");
        _npcTexture = _game.CreateTexture("assets/sprites/npcs/npc.png");
        _projectileTexture = _game.CreateTexture("assets/sprites/projectiles/projectile.png");

        // Load ground/tiles
        _groundTexture = _game.CreateTexture("assets/sprites/tiles/ground_grid.png");

        // Load UI textures
        _healthBgTexture = _game.CreateTexture("assets/sprites/ui/health_bg.png");
        _healthFillTexture = _game.CreateTexture("assets/sprites/ui/health_fill.png");
        _dialogueBoxTexture = _game.CreateTexture("assets/sprites/ui/dialogue_box.png");
        _arrowTexture = _game.CreateTexture("assets/sprites/ui/arrow.png");
        _titleTexture = _game.CreateTexture("assets/sprites/ui/title.png");
        _pressStartTexture = _game.CreateTexture("assets/sprites/ui/press_start.png");
    }

    private void CreateNPC()
    {
        var npc = new NPC(_game);
        npc.Initialize();
        // Position NPC in a visible spot (right side of screen)
        npc.Setup(_npcTexture, 600, 300, "Village Elder", DialogueFactory.CreateSampleDialogue());
        _npcs.Add(npc);
    }

    private void SpawnEnemy()
    {
        if (_enemies.Count >= MaxEnemies) return;

        var enemy = new Enemy(_game);
        enemy.Initialize();

        // Spawn at random edge of screen
        float x, y;
        int edge = Random.Shared.Next(4);
        switch (edge)
        {
            case 0: // Top
                x = Random.Shared.Next(50, ScreenWidth - 50);
                y = 50;
                break;
            case 1: // Right
                x = ScreenWidth - 50;
                y = Random.Shared.Next(50, ScreenHeight - 50);
                break;
            case 2: // Bottom
                x = Random.Shared.Next(50, ScreenWidth - 50);
                y = ScreenHeight - 50;
                break;
            default: // Left
                x = 50;
                y = Random.Shared.Next(50, ScreenHeight - 50);
                break;
        }

        enemy.Setup(_combatSystem, _player, _enemyTexture, x, y);
        _enemies.Add(enemy);

        _game.GameLog($"Enemy spawned at ({x:F0}, {y:F0})");
    }

    public void Start()
    {
        // Hide all gameplay elements for title screen
        HideGameplayElements();

        _stateManager.SetState(GameState.Title);
        _startCooldown = 0.5f; // Prevent immediate start
        _game.GameLog("Game started - showing title screen");
        _game.GameLog("Press SPACE to start");
    }

    private void HideGameplayElements()
    {
        // Hide player
        _game.UpdateSprite(new SpriteUpdateDto
        {
            id = _player.SpriteId,
            scale_x = 0,
            scale_y = 0
        });

        // Hide NPCs
        foreach (var npc in _npcs)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = npc.SpriteId,
                scale_x = 0,
                scale_y = 0
            });
        }

        // Hide ground
        _game.UpdateSprite(new SpriteUpdateDto
        {
            id = _groundSpriteId,
            scale_x = 0,
            scale_y = 0
        });
    }

    private void ShowGameplayElements()
    {
        // Show player
        _game.UpdateSprite(new SpriteUpdateDto
        {
            id = _player.SpriteId,
            scale_x = 1,
            scale_y = 1
        });

        // Show NPCs
        foreach (var npc in _npcs)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = npc.SpriteId,
                scale_x = 1,
                scale_y = 1
            });
        }

        // Show ground
        _game.UpdateSprite(new SpriteUpdateDto
        {
            id = _groundSpriteId,
            scale_x = 1,
            scale_y = 1
        });
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

        // Update UI
        _uiManager.Update(deltaTime, _stateManager.CurrentState);
    }

    private void HandleGlobalInput()
    {
        // ESC to return to title from any state
        if (_game.IsKeyPressed(Keys.Escape)) // ESC
        {
            if (_stateManager.CurrentState == GameState.Playing)
            {
                HideGameplayElements();
                _stateManager.SetState(GameState.Title);
            }
            else if (_stateManager.CurrentState == GameState.Dialogue)
            {
                _dialogueSystem.EndDialogue();
            }
        }
    }

    private void UpdateTitleScreen(float deltaTime)
    {
        // Space to start game
        if (_game.IsKeyPressed(Keys.Space) && _startCooldown <= 0) // Space
        {
            StartGame();
        }
    }

    private void StartGame()
    {
        // Show gameplay elements
        ShowGameplayElements();

        // Reset player
        _player.Reset();

        // Clear existing enemies
        foreach (var enemy in _enemies)
        {
            enemy.Destroy();
        }
        _enemies.Clear();
        _combatSystem.ClearEnemies();
        _combatSystem.Clear();

        // Spawn initial enemy
        SpawnEnemy();

        _enemySpawnTimer = EnemySpawnInterval;
        _stateManager.SetState(GameState.Playing);

        _game.GameLog("=== GAME STARTED ===");
        _game.GameLog("WASD to move, LMB to attack, RMB to shoot");
        _game.GameLog("E to talk to NPCs");
    }

    private void UpdatePlaying(float deltaTime)
    {
        // Update player
        _player.Update(deltaTime);

        // Update enemies
        for (int i = _enemies.Count - 1; i >= 0; i--)
        {
            var enemy = _enemies[i];
            enemy.Update(deltaTime);

            // Remove dead enemies after death animation
            if (enemy.Health.IsDead && enemy.IsDeathAnimationComplete)
            {
                enemy.Destroy();
                _enemies.RemoveAt(i);
            }
        }

        // Update NPCs
        foreach (var npc in _npcs)
        {
            npc.Update(deltaTime);
        }

        // Update combat system
        _combatSystem.Update(deltaTime);

        // Check for NPC interaction
        CheckNPCInteraction();

        // Enemy spawning
        _enemySpawnTimer -= deltaTime;
        if (_enemySpawnTimer <= 0)
        {
            SpawnEnemy();
            _enemySpawnTimer = EnemySpawnInterval;
        }

        // Update camera to follow player
        UpdateCamera();
    }

    private void CheckNPCInteraction()
    {
        _nearbyNpc = null;

        foreach (var npc in _npcs)
        {
            if (npc.CanInteract(_player))
            {
                _nearbyNpc = npc;
                break;
            }
        }

        // E to interact
        if (_nearbyNpc != null && _game.IsKeyPressed(Keys.E) && _interactionCooldown <= 0)
        {
            StartDialogue(_nearbyNpc);
            _interactionCooldown = 0.5f;
        }
    }

    private void StartDialogue(NPC npc)
    {
        var dialogue = npc.GetDialogue();
        if (dialogue == null)
        {
            _game.GameLog($"{npc.Name} has nothing to say.");
            return;
        }

        npc.SetTalking(true);
        _dialogueSystem.StartDialogue(dialogue);
        _stateManager.SetState(GameState.Dialogue);
    }

    private void UpdateDialogue(float deltaTime)
    {
        _dialogueSystem.Update(deltaTime);
    }

    private void OnDialogueEnded()
    {
        foreach (var npc in _npcs)
        {
            npc.SetTalking(false);
        }

        if (_stateManager.CurrentState == GameState.Dialogue)
        {
            _stateManager.SetState(GameState.Playing);
        }
    }

    private void UpdateGameOver(float deltaTime)
    {
        // R to restart
        if (_game.IsKeyPressed(Keys.R)) // R
        {
            StartGame();
        }

        // Space to return to title
        if (_game.IsKeyPressed(Keys.Space) && _startCooldown <= 0) // Space
        {
            HideGameplayElements();
            _stateManager.SetState(GameState.Title);
            _startCooldown = 0.5f;
        }
    }

    private void UpdateCamera()
    {
        // Center camera on player
        float camX = _player.X - ScreenWidth / 2 + _player.Width / 2;
        float camY = _player.Y - ScreenHeight / 2 + _player.Height / 2;

        // Clamp camera to level bounds (for now, just center on origin)
        camX = 0;
        camY = 0;

        _game.SetCameraPosition(camX, camY);
    }

    private void OnPlayerDeath()
    {
        _game.GameLog("=== GAME OVER ===");
        _game.GameLog("Press R to restart or SPACE for title");
        _stateManager.SetState(GameState.GameOver);
        _startCooldown = 0.5f;
    }

    private void OnPlayerHealthChanged(int current, int max)
    {
        _uiManager.UpdateHealthBar((float)current / max);
    }

    private void OnEntityKilled(EntityBase entity)
    {
        if (entity is Enemy enemy)
        {
            _game.GameLog("Enemy defeated!");
        }
    }

    private void HandleStateChanged(GameState oldState, GameState newState)
    {
        _game.GameLog($"State: {oldState} -> {newState}");
    }
}
