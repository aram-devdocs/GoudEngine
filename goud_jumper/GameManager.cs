// GameManager.cs

using System;
using System.Collections.Generic;
using CsBindgen;

public class GameManager
{
    private GoudGame game;
    private uint PlayerTextureId;
    private uint PlayerSpriteId;
    private AnimationService animationService;
    private PlayerStateMachine playerStateMachine;

    public GameManager(GoudGame game)
    {
        this.game = game;
        this.animationService = new AnimationService();
        this.playerStateMachine = new PlayerStateMachine();
    }

    public void Initialize()
    {
        // p1_duck = 365 98 69 71
        // p1_front = 0 196 66 92
        // p1_hurt = 438 0 69 92
        // p1_jump = 438 93 67 94
        // p1_stand = 67 196 66 92
        // p1_walk01 = 0 0 72 97
        // p1_walk02 = 73 0 72 97
        // p1_walk03 = 146 0 72 97
        // p1_walk04 = 0 98 72 97
        // p1_walk05 = 73 98 72 97
        // p1_walk06 = 146 98 72 97
        // p1_walk07 = 219 0 72 97
        // p1_walk08 = 292 0 72 97
        // p1_walk09 = 219 98 72 97
        // p1_walk10 = 365 0 72 97
        // p1_walk11 = 292 98 72 97

        PlayerTextureId = game.CreateTexture("assets/p1_spritesheet.png");
        PlayerSpriteId = game.AddSprite(
            new SpriteCreateDto
            {
                x = 0,
                y = 0,
                z_layer = 0,
                texture_id = PlayerTextureId,
                frame = new Rectangle
                {
                    x = 67,
                    y = 196,
                    width = 66,
                    height = 92
                }
            }
        );
    }

    public void Start() { }

    public void Update(float deltaTime)
    {
        HandleInput();
        animationService.UpdateAnimation(game, PlayerSpriteId, playerStateMachine.CurrentState);
    }

    private void HandleInput()
    {
        bool isMoving = false;

        if (game.IsKeyPressed(32)) // Key.Space
        {
            playerStateMachine.SetState(PlayerState.Jumping);
            isMoving = true;
        }
        else if (game.IsKeyPressed(83)) // Key.S
        {
            playerStateMachine.SetState(PlayerState.Ducking);
            isMoving = true;
        }
        else if (game.IsKeyPressed(65)) // Key.A
        {
            playerStateMachine.SetState(PlayerState.Walking);
            isMoving = true;
        }
        else if (game.IsKeyPressed(68)) // Key.D
        {
            playerStateMachine.SetState(PlayerState.Walking);
            isMoving = true;
        }

        if (!isMoving)
        {
            playerStateMachine.SetState(PlayerState.Standing);
        }
    }

    private void ResetGame() { }
}

public enum PlayerState
{
    Standing,
    Walking,
    Jumping,
    Ducking,
    Hurt
}

public class PlayerStateMachine
{
    public PlayerState CurrentState { get; private set; }

    public PlayerStateMachine()
    {
        CurrentState = PlayerState.Standing;
    }

    public void SetState(PlayerState newState)
    {
        CurrentState = newState;
    }
}

public class AnimationService
{
    private Dictionary<PlayerState, Rectangle> stateToFrameMap;

    public AnimationService()
    {
        stateToFrameMap = new Dictionary<PlayerState, Rectangle>
        {
            {
                PlayerState.Standing,
                new Rectangle
                {
                    x = 67,
                    y = 196,
                    width = 66,
                    height = 92
                }
            },
            {
                PlayerState.Walking,
                new Rectangle
                {
                    x = 0,
                    y = 0,
                    width = 72,
                    height = 97
                }
            },
            {
                PlayerState.Jumping,
                new Rectangle
                {
                    x = 438,
                    y = 93,
                    width = 67,
                    height = 94
                }
            },
            {
                PlayerState.Ducking,
                new Rectangle
                {
                    x = 365,
                    y = 98,
                    width = 69,
                    height = 71
                }
            },
            {
                PlayerState.Hurt,
                new Rectangle
                {
                    x = 438,
                    y = 0,
                    width = 69,
                    height = 92
                }
            }
        };
    }

    public void UpdateAnimation(GoudGame game, uint spriteId, PlayerState state)
    {
        var frame = stateToFrameMap[state];
        game.UpdateSprite(
            new SpriteUpdateDto
            {
                id = spriteId,
                frame = frame,
                // debug = true
            }
        );
    }
}
