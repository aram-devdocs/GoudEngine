using System;
using System.Collections.Generic;
using CsBindgen;

public class GameManager
{
    private GoudGame game;
    private uint PlayerSpriteId;

    private PlayerStateMachine playerStateMachine;
    private AnimationController? animationController;

    private bool IsGoingLeft = false;

    // goud_jumper/assets/1 Pink_Monster/Pink_Monster_Attack1_4.png
    // goud_jumper/assets/1 Pink_Monster/Pink_Monster_Attack1_4.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Attack2_6.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Climb_4.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Death_8.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Hurt_4.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Idle_4.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Jump_8.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Push_6.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Run_6.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Throw_4.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Walk_6.png goud_jumper/assets/1 Pink_Monster/Pink_Monster_Walk+Attack_6.png

    public GameManager(GoudGame game)
    {
        this.game = game;
        this.playerStateMachine = new PlayerStateMachine();
    }

    public void Initialize()
    {
        var stateToTextureMap = new Dictionary<string, (string texturePath, int frameCount, int frameWidth, int frameHeight)>
        {
            { PlayerState.Attack1.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Attack1_4.png", 4, 32, 32) },
            { PlayerState.Attack2.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Attack2_6.png", 6, 32, 32) },
            { PlayerState.Climb.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Climb_4.png", 4, 32, 32) },
            { PlayerState.Death.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Death_8.png", 8, 32, 32) },
            { PlayerState.Hurt.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Hurt_4.png", 4, 32, 32) },
            { PlayerState.Idle.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Idle_4.png", 4, 32, 32) },
            { PlayerState.Jumping.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Jump_8.png", 8, 32, 32) },
            { PlayerState.Push.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Push_6.png", 6, 32, 32) },
            { PlayerState.Run.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Run_6.png", 6, 32, 32) },
            { PlayerState.Throw.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Throw_4.png", 4, 32, 32) },
            { PlayerState.Walking.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Walk_6.png", 6, 32, 32) },
            { PlayerState.WalkAttack.ToString(), ("assets/1 Pink_Monster/Pink_Monster_Walk+Attack_6.png", 6, 32, 32) }
        };

        this.animationController = new AnimationController(game, stateToTextureMap);

        // Initialize PlayerSpriteId with the first state's texture
        var initialTextureId = animationController.GetInitialTextureId(PlayerState.Idle.ToString());
        PlayerSpriteId = game.AddSprite(
            new SpriteCreateDto
            {
                x = 0,
                y = 0,
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
                    id = PlayerSpriteId,
                    frame = frame,
                    texture_id = textureId
                }
            );
        }
    }

    private void HandleInput()
    {
        bool isMoving = false;

        if (game.IsKeyPressed(32)) // Key.Space
        {
            playerStateMachine.SetState(PlayerState.Jumping);
            isMoving = true;
        }
        else if (game.IsKeyPressed(65)) // Key.A
        {
            playerStateMachine.SetState(PlayerState.Walking);
            isMoving = true;
            IsGoingLeft = true;
        }
        else if (game.IsKeyPressed(68)) // Key.D
        {
            playerStateMachine.SetState(PlayerState.Walking);
            isMoving = true;
            IsGoingLeft = false;
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
    WalkAttack
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
        CurrentState = newState;
    }
}
