using System;
using System.Collections.Generic;
using CsBindgen;

public class GameManager
{
    private GoudGame game;
    private uint PlayerTextureId;
    private uint PlayerSpriteId;

    private PlayerStateMachine playerStateMachine;
    private AnimationController animationController;

    private bool IsGoingLeft = false;

    public GameManager(GoudGame game)
    {
        this.game = game;
        this.playerStateMachine = new PlayerStateMachine();

        const int DefaultPlayerWidth = 0;
        const int DefaultPlayerHeight = 0;

        var stateToPositionMap = new Dictionary<
            string,
            List<(int x, int y, int? width, int? height)>
        >
        {
            // tdoo: fill this in
        };
        this.animationController = new AnimationController(
            DefaultPlayerWidth,
            DefaultPlayerHeight,
            stateToPositionMap
        );
    }

    public void Initialize()
    {
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
        var frame = animationController.GetFrame(
            playerStateMachine.CurrentState.ToString(),
            deltaTime
        );
        game.UpdateSprite(
            new SpriteUpdateDto
            {
                id = PlayerSpriteId,
                frame = frame,
                scale_x = IsGoingLeft ? -1 : 1
            }
        );
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
