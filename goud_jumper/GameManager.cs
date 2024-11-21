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

    public GameManager(GoudGame game)
    {
        this.game = game ?? throw new ArgumentNullException(nameof(game));
        this.playerStateMachine = new PlayerStateMachine();
    }

    public void Initialize()
    {
        Console.WriteLine("Tilemap");

        var tileset_texture_id = game.CreateTexture(
            "../goud_engine/src/libs/platform/graphics/rendering/tiled/_Tiled/Tilesets/Tileset.png"
        );
        var tiled_id = game.LoadTiledMap(
            "Map",
            "../goud_engine/src/libs/platform/graphics/rendering/tiled/_Tiled/Maps/Map.tmx",
            tileset_texture_id
        );

        game.SetSelectedTiledMapById(tiled_id);
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
                    x = 100,
                    y = 100,
                    scale_x = isGoingLeft ? -2 : 2,
                }
            );
        }
    }

    private void HandleInput()
    {
        bool isMoving = false;

        if (game.IsKeyPressed(32)) // Space key
        {
            playerStateMachine.SetState(PlayerState.Jumping);
            isMoving = true;
        }
        else if (game.IsKeyPressed(65)) // 'A' key
        {
            playerStateMachine.SetState(PlayerState.Walking);
            isMoving = true;
            isGoingLeft = true;
        }
        else if (game.IsKeyPressed(68)) // 'D' key
        {
            playerStateMachine.SetState(PlayerState.Walking);
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

        if (!isMoving)
        {
            playerStateMachine.SetState(PlayerState.Idle);
        }
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
