using GoudEngine;
using IsometricRpg.Core;

namespace IsometricRpg.Player;

/// <summary>
/// Handles player input and movement in 8 directions for isometric gameplay.
/// </summary>
public class PlayerController
{
    private readonly GoudGame _game;
    private readonly Player _player;

    // Movement configuration
    public float MoveSpeed { get; set; } = 150f;

    // Input state
    private float _inputX;
    private float _inputY;
    private bool _isMoving;

    // Key codes (GLFW)
    private const int KeyW = 87;
    private const int KeyA = 65;
    private const int KeyS = 83;
    private const int KeyD = 68;

    public PlayerController(GoudGame game, Player player)
    {
        _game = game;
        _player = player;
    }

    /// <summary>
    /// Handle input and update player movement.
    /// Returns true if player is moving.
    /// </summary>
    public bool HandleInput(float deltaTime)
    {
        _inputX = 0;
        _inputY = 0;

        // WASD movement input
        if (_game.IsKeyPressed(KeyW)) _inputY -= 1;
        if (_game.IsKeyPressed(KeyS)) _inputY += 1;
        if (_game.IsKeyPressed(KeyA)) _inputX -= 1;
        if (_game.IsKeyPressed(KeyD)) _inputX += 1;

        _isMoving = _inputX != 0 || _inputY != 0;

        if (_isMoving)
        {
            // Update facing direction
            int direction = IsometricUtils.GetDirectionFromInput(_inputX, _inputY);
            if (direction >= 0)
            {
                _player.Direction = direction;
            }

            // Apply movement with isometric adjustment
            ApplyMovement(deltaTime);
        }

        return _isMoving;
    }

    /// <summary>
    /// Apply isometric movement based on input direction.
    /// </summary>
    private void ApplyMovement(float deltaTime)
    {
        // Normalize diagonal movement
        float magnitude = MathF.Sqrt(_inputX * _inputX + _inputY * _inputY);
        if (magnitude > 1f)
        {
            _inputX /= magnitude;
            _inputY /= magnitude;
        }

        // Apply movement
        float moveAmount = MoveSpeed * deltaTime;
        _player.X += _inputX * moveAmount;
        _player.Y += _inputY * moveAmount;

        // Clamp to screen bounds (with padding for sprite size)
        float padding = 16f;
        _player.X = Math.Clamp(_player.X, padding, GameManager.ScreenWidth - _player.Width - padding);
        _player.Y = Math.Clamp(_player.Y, padding, GameManager.ScreenHeight - _player.Height - padding);
    }

    /// <summary>
    /// Get current input direction vector (normalized).
    /// </summary>
    public (float x, float y) GetInputDirection()
    {
        if (!_isMoving) return (0, 0);

        float magnitude = MathF.Sqrt(_inputX * _inputX + _inputY * _inputY);
        if (magnitude < 0.001f) return (0, 0);

        return (_inputX / magnitude, _inputY / magnitude);
    }

    /// <summary>
    /// Check if player is currently moving.
    /// </summary>
    public bool IsMoving => _isMoving;
}
