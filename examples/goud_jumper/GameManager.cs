using System;
using System.Collections.Generic;
using CsBindgen;

public class GameManager
{
    private readonly GoudGame game;
    private uint playerSpriteId;

    private readonly PlayerStateMachine playerStateMachine;
    private AnimationController? animationController;

    private bool isGoingLeft = false;

    private uint tilemapId;
    private uint tilemapId1;

    private float playerX = 100;
    private float playerY = 100;
    private float playerVelocityY = 0;
    private const float gravity = 9.8f;
    private const float jumpStrength = 5.0f;
    private const float moveSpeed = 2.0f;

    private float cameraX = 0;
    private float cameraY = 0;
    private float cameraZoom = 1.0f;

    public GameManager(GoudGame game)
    {
        this.game = game ?? throw new ArgumentNullException(nameof(game));
        this.playerStateMachine = new PlayerStateMachine();
    }

    public void Initialize()
    {
        Console.WriteLine("Tilemap");

        var tileset_texture_id = game.CreateTexture("assets/tiled/Tileset.png");
        tilemapId = game.LoadTiledMap("Map", "assets/tiled/Map.tmx", [tileset_texture_id]);

        tilemapId1 = game.LoadTiledMap("Map1", "assets/tiled/Map1.tmx", [tileset_texture_id]);

        game.SetSelectedTiledMapById(tilemapId);
        var stateConfigurations = new Dictionary<string, AnimationStateConfig>
        {
            // Grid-based configurations
            {
                PlayerState.Attack1.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Attack1_4.png",
                    32,
                    32,
                    0.1f,
                    1.5f,
                    false,
                    0,
                    4,
                    4
                )
            },
            {
                PlayerState.Attack2.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Attack2_6.png",
                    32,
                    32,
                    0.1f,
                    1.5f,
                    false,
                    0,
                    6,
                    6
                )
            },
            {
                PlayerState.Climb.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Climb_4.png",
                    32,
                    32,
                    0.15f,
                    1.0f,
                    true,
                    0,
                    4,
                    4
                )
            },
            {
                PlayerState.Death.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Death_8.png",
                    32,
                    32,
                    0.1f,
                    1.0f,
                    false,
                    0,
                    8,
                    8
                )
            },
            {
                PlayerState.Hurt.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Hurt_4.png",
                    32,
                    32,
                    0.1f,
                    1.0f,
                    false,
                    0,
                    4,
                    4
                )
            },
            {
                PlayerState.Idle.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Idle_4.png",
                    32,
                    32,
                    0.2f,
                    0.5f,
                    true,
                    0,
                    4,
                    4
                )
            },
            {
                PlayerState.Jumping.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Jump_8.png",
                    32,
                    32,
                    0.1f,
                    1.2f,
                    false,
                    0,
                    8,
                    8
                )
            },
            {
                PlayerState.Push.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Push_6.png",
                    32,
                    32,
                    0.1f,
                    1.0f,
                    true,
                    0,
                    6,
                    6
                )
            },
            {
                PlayerState.Run.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Run_6.png",
                    32,
                    32,
                    0.08f,
                    1.2f,
                    true,
                    0,
                    6,
                    6
                )
            },
            {
                PlayerState.Throw.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Throw_4.png",
                    32,
                    32,
                    0.1f,
                    1.5f,
                    false,
                    0,
                    4,
                    4
                )
            },
            {
                PlayerState.Walking.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Walk_6.png",
                    32,
                    32,
                    0.1f,
                    1.0f,
                    true,
                    0,
                    6,
                    6
                )
            },
            {
                PlayerState.WalkAttack.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Walk+Attack_6.png",
                    32,
                    32,
                    0.1f,
                    1.3f,
                    false,
                    0,
                    6,
                    6
                )
            },
            {
                PlayerState.Custom1.ToString(),
                new AnimationStateConfig(
                    "assets/samuri_animations.png",
                    new List<Rectangle>
                    {
                        new Rectangle
                        {
                            x = 0,
                            y = 0,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 64,
                            y = 0,
                            width = 64,
                            height = 48
                        },
                    },
                    frameTime: 0.1f,
                    speedScale: 1.0f,
                    shouldLoop: true
                )
            },
            {
                PlayerState.Custom2.ToString(),
                new AnimationStateConfig(
                    "assets/samuri_animations.png",
                    new List<Rectangle>
                    {
                        new Rectangle
                        {
                            x = 0,
                            y = 48,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 64,
                            y = 48,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 128,
                            y = 48,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 192,
                            y = 48,
                            width = 64,
                            height = 48
                        },
                    },
                    frameTime: 0.1f,
                    speedScale: 1.0f,
                    shouldLoop: true
                )
            },
            {
                PlayerState.Custom3.ToString(),
                new AnimationStateConfig(
                    "assets/samuri_animations.png",
                    new List<Rectangle>
                    {
                        new Rectangle
                        {
                            x = 0,
                            y = 96,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 64,
                            y = 96,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 128,
                            y = 96,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 192,
                            y = 96,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 256,
                            y = 96,
                            width = 64,
                            height = 48
                        },
                    },
                    frameTime: 0.1f,
                    speedScale: 0.8f,
                    shouldLoop: true
                )
            },
        };

        this.animationController = new AnimationController(game, stateConfigurations);

        // Initialize playerSpriteId with the initial state's texture
        var initialTextureId = animationController.GetInitialTextureId(PlayerState.Idle.ToString());
        playerSpriteId = game.AddSprite(
            new SpriteCreateDto
            {
                z_layer = 0,
                scale_x = 2,
                scale_y = 2,
                texture_id = initialTextureId,
            }
        );
    }

    public void Start() { }

    public void Update(float deltaTime)
    {
        HandleInput();
        UpdatePlayerPosition(deltaTime);
        if (animationController != null)
        {
            var (frame, textureId) = animationController.GetFrame(
                playerStateMachine.CurrentState.ToString(),
                deltaTime
            );
            game.UpdateSprite(
                new SpriteUpdateDto
                {
                    id = playerSpriteId,
                    frame = frame,
                    texture_id = textureId,
                    x = playerX,
                    y = playerY,
                    scale_x = isGoingLeft ? -2 : 2,
                }
            );
        }
    }

    private void UpdatePlayerPosition(float deltaTime)
    {
        playerVelocityY += gravity * deltaTime;
        playerY += playerVelocityY;

        if (playerY > 100) // Assuming ground level is at y = 100
        {
            playerY = 100;
            playerVelocityY = 0;
            playerStateMachine.SetState(PlayerState.Idle);
        }
    }

    public void HandleInput()
    {
        bool isMoving = false;

        if (game.IsKeyPressed(32) && playerY == 100) // Space key and on ground
        {
            playerStateMachine.SetState(PlayerState.Jumping);
            playerVelocityY = -jumpStrength;
            isMoving = true;
        }
        else if (game.IsKeyPressed(65)) // 'A' key
        {
            playerStateMachine.SetState(PlayerState.Walking);
            playerX -= moveSpeed;
            isMoving = true;
            isGoingLeft = true;
        }
        else if (game.IsKeyPressed(68)) // 'D' key
        {
            playerStateMachine.SetState(PlayerState.Walking);
            playerX += moveSpeed;
            isMoving = true;
            isGoingLeft = false;
        }
        // c + 1
        else if (game.IsKeyPressed(67) && game.IsKeyPressed(49))
        {
            playerStateMachine.SetState(PlayerState.Custom1);
            isMoving = true;
        }
        // c + 2
        else if (game.IsKeyPressed(67) && game.IsKeyPressed(50))
        {
            playerStateMachine.SetState(PlayerState.Custom2);
            isMoving = true;
        }
        // c + 3
        else if (game.IsKeyPressed(67) && game.IsKeyPressed(51))
        {
            playerStateMachine.SetState(PlayerState.Custom3);
            isMoving = true;
        }

        // Change tile maps

        // m + 0
        if (game.IsKeyPressed(77) && game.IsKeyPressed(48))
        {
            game.SetSelectedTiledMapById(tilemapId);
        }

        // m + 1
        if (game.IsKeyPressed(77) && game.IsKeyPressed(49))
        {
            game.SetSelectedTiledMapById(tilemapId1);
        }

        if (!isMoving && playerY == 100)
        {
            playerStateMachine.SetState(PlayerState.Idle);
        }

        // handle camera


        // Camera controls
        if (game.IsKeyPressed(265)) // Up arrow key
        {
            cameraY -= 10.0f;
        }
        if (game.IsKeyPressed(264)) // Down arrow key
        {
            cameraY += 10.0f;
        }
        if (game.IsKeyPressed(263)) // Left arrow key
        {
            cameraX -= 10.0f;
        }
        if (game.IsKeyPressed(262)) // Right arrow key
        {
            cameraX += 10.0f;
        }
        if (game.IsKeyPressed(79)) // 'O' key
        {
            cameraZoom += 0.01f;
        }
        if (game.IsKeyPressed(80)) // 'P' key
        {
            cameraZoom -= 0.01f;
        }

        game.SetCameraPosition(cameraX, cameraY);
        game.SetCameraZoom(cameraZoom);
    }

    private void ResetGame() { }
}

public enum PlayerState
{
    Walking,
    Jumping,
    Ducking,
    Hurt,
    Attack1,
    Attack2,
    Climb,
    Death,
    Idle,
    Push,
    Run,
    Throw,
    WalkAttack,
    Custom1,
    Custom2,
    Custom3
}

public class PlayerStateMachine
{
    public PlayerState CurrentState { get; private set; }

    public PlayerStateMachine()
    {
        CurrentState = PlayerState.Idle;
    }

    public void SetState(PlayerState newState)
    {
        if (CurrentState != newState)
        {
            CurrentState = newState;
        }
    }
}
