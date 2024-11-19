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
        var stateConfigurations = new Dictionary<string, AnimationStateConfig>
        {
            // Grid-based configurations
            {
                PlayerState.Attack1.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Attack1_4.png",
                    4,
                    32,
                    32,
                    0.1f,
                    1.5f,
                    false
                )
            },
            {
                PlayerState.Attack2.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Attack2_6.png",
                    6,
                    32,
                    32,
                    0.1f,
                    1.5f,
                    false
                )
            },
            {
                PlayerState.Climb.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Climb_4.png",
                    4,
                    32,
                    32,
                    0.15f,
                    1.0f,
                    true
                )
            },
            {
                PlayerState.Death.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Death_8.png",
                    8,
                    32,
                    32,
                    0.1f,
                    1.0f,
                    false
                )
            },
            {
                PlayerState.Hurt.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Hurt_4.png",
                    4,
                    32,
                    32,
                    0.1f,
                    1.0f,
                    false
                )
            },
            {
                PlayerState.Idle.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Idle_4.png",
                    4,
                    32,
                    32,
                    0.2f,
                    0.5f,
                    true
                )
            },
            {
                PlayerState.Jumping.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Jump_8.png",
                    8,
                    32,
                    32,
                    0.1f,
                    1.2f,
                    false
                )
            },
            {
                PlayerState.Push.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Push_6.png",
                    6,
                    32,
                    32,
                    0.1f,
                    1.0f,
                    true
                )
            },
            {
                PlayerState.Run.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Run_6.png",
                    6,
                    32,
                    32,
                    0.08f,
                    1.2f,
                    true
                )
            },
            {
                PlayerState.Throw.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Throw_4.png",
                    4,
                    32,
                    32,
                    0.1f,
                    1.5f,
                    false
                )
            },
            {
                PlayerState.Walking.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Walk_6.png",
                    6,
                    32,
                    32,
                    0.1f,
                    1.0f,
                    true
                )
            },
            {
                PlayerState.WalkAttack.ToString(),
                new AnimationStateConfig(
                    "assets/1 Pink_Monster/Pink_Monster_Walk+Attack_6.png",
                    6,
                    32,
                    32,
                    0.1f,
                    1.3f,
                    false
                )
            },
            // Frame list-based configuration for a custom state
            {
                PlayerState.Custom.ToString(),
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
                        new Rectangle
                        {
                            x = 128,
                            y = 0,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 192,
                            y = 0,
                            width = 64,
                            height = 48
                        },
                        new Rectangle
                        {
                            x = 256,
                            y = 0,
                            width = 64,
                            height = 48
                        },

                        // Additional frames
                    },
                    frameTime: 0.1f,
                    speedScale: 1.0f,
                    shouldLoop: false
                )
            }
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
                    // flip_x = isGoingLeft
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
        else if (game.IsKeyPressed(67)) // 'C' key for custom animation
        {
            playerStateMachine.SetState(PlayerState.Custom);
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
    Custom // New state for complex spritesheet
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
