using GoudEngine;
using CsBindgen;
using IsometricRpg.Core;

namespace IsometricRpg.NPCs;

/// <summary>
/// A non-player character that can be interacted with.
/// </summary>
public class NPC : EntityBase
{
    // NPC properties
    public string Name { get; set; } = "NPC";
    public Dialogue? Dialogue { get; set; }

    // Interaction settings
    public float InteractionRange { get; set; } = 50f;

    // Visual indicator when player is in range
    private uint _indicatorSpriteId;
    private bool _showIndicator;

    // Animation
    private AnimationController? _animationController;
    private bool _isTalking;

    public NPC(GoudGame game) : base(game)
    {
        Width = 32;
        Height = 32;
    }

    public override void Initialize()
    {
        // NPCs are static entities, just need sprite setup
    }

    /// <summary>
    /// Set up NPC with texture and position.
    /// </summary>
    public void Setup(uint textureId, float x, float y, string name, Dialogue? dialogue = null)
    {
        Name = name;
        Dialogue = dialogue;
        X = x;
        Y = y;

        CreateSprite(textureId, x, y, 10);
    }

    /// <summary>
    /// Set up animations for the NPC.
    /// </summary>
    public void SetupAnimations(Dictionary<string, AnimationStateConfig> animations)
    {
        _animationController = new AnimationController(Game, animations);
    }

    public override void Update(float deltaTime)
    {
        if (!IsActive) return;

        // Update animation if available
        UpdateAnimation(deltaTime);
    }

    /// <summary>
    /// Update animation frame.
    /// </summary>
    private void UpdateAnimation(float deltaTime)
    {
        if (_animationController == null) return;

        string stateName = _isTalking ? "Talking" : "Idle";

        try
        {
            var (frame, textureId) = _animationController.GetFrame(stateName, deltaTime);

            Game.UpdateSprite(new SpriteUpdateDto
            {
                id = SpriteId,
                texture_id = textureId,
                source_rect = frame,
                x = X,
                y = Y,
                z_layer = ZLayer
            });
        }
        catch (ArgumentException)
        {
            // Animation state not found
        }
    }

    /// <summary>
    /// Check if a position is within interaction range.
    /// </summary>
    public bool IsInInteractionRange(float x, float y)
    {
        float distance = IsometricUtils.Distance(X + Width / 2, Y + Height / 2, x, y);
        return distance <= InteractionRange;
    }

    /// <summary>
    /// Check if player entity is within interaction range.
    /// </summary>
    public bool CanInteract(EntityBase player)
    {
        var (px, py) = player.GetCenter();
        return IsInInteractionRange(px, py);
    }

    /// <summary>
    /// Set talking state (for animation).
    /// </summary>
    public void SetTalking(bool talking)
    {
        _isTalking = talking;
    }

    /// <summary>
    /// Show/hide interaction indicator.
    /// </summary>
    public void ShowIndicator(bool show, uint? indicatorTextureId = null)
    {
        if (show == _showIndicator) return;

        _showIndicator = show;

        if (show && indicatorTextureId.HasValue)
        {
            // Create indicator sprite above NPC
            _indicatorSpriteId = Game.AddSprite(new SpriteCreateDto
            {
                texture_id = indicatorTextureId.Value,
                x = X + Width / 2 - 8, // Center above NPC
                y = Y - 20,
                z_layer = ZLayer + 1,
                scale_x = 0.5f,
                scale_y = 0.5f
            });
        }
        else if (!show && _indicatorSpriteId != 0)
        {
            Game.RemoveSprite(_indicatorSpriteId);
            _indicatorSpriteId = 0;
        }
    }

    /// <summary>
    /// Get the NPC's dialogue.
    /// </summary>
    public Dialogue? GetDialogue()
    {
        return Dialogue;
    }

    public override void Destroy()
    {
        if (_indicatorSpriteId != 0)
        {
            Game.RemoveSprite(_indicatorSpriteId);
            _indicatorSpriteId = 0;
        }
        base.Destroy();
    }
}
