using GoudEngine;
using CsBindgen;
using IsometricRpg.NPCs;

namespace IsometricRpg.UI;

/// <summary>
/// UI for displaying dialogue boxes and choices.
/// Since the engine doesn't have text rendering, this uses sprite-based UI.
/// </summary>
public class DialogueUI
{
    private readonly GoudGame _game;
    private readonly DialogueSystem _dialogueSystem;

    // Sprite IDs
    private uint _dialogueBoxSpriteId;
    private uint _selectionArrowSpriteId;
    private readonly List<uint> _choiceSprites = new();

    // Layout settings
    private const float BoxX = 50;
    private const float BoxY = 400;
    private const float BoxWidth = 700;
    private const float BoxHeight = 150;
    private const float ChoiceStartY = 450;
    private const float ChoiceSpacing = 30;
    private const float ArrowOffsetX = -20;

    // State
    private bool _isVisible;

    // Z-layers
    private const int DialogueZLayer = 110;

    public DialogueUI(GoudGame game, DialogueSystem dialogueSystem)
    {
        _game = game;
        _dialogueSystem = dialogueSystem;

        // Subscribe to dialogue events
        dialogueSystem.OnDialogueStarted += OnDialogueStarted;
        dialogueSystem.OnDialogueEnded += OnDialogueEnded;
        dialogueSystem.OnDialogueNodeChanged += OnNodeChanged;
        dialogueSystem.OnChoiceSelectionChanged += OnSelectionChanged;
    }

    /// <summary>
    /// Initialize with textures.
    /// </summary>
    public void Initialize(uint dialogueBoxTexture, uint arrowTexture)
    {
        // Create dialogue box (hidden initially - scale 0)
        _dialogueBoxSpriteId = _game.AddSprite(new SpriteCreateDto
        {
            texture_id = dialogueBoxTexture,
            x = BoxX,
            y = BoxY,
            z_layer = DialogueZLayer,
            scale_x = 0,
            scale_y = 0
        });

        // Create selection arrow (hidden initially - scale 0)
        _selectionArrowSpriteId = _game.AddSprite(new SpriteCreateDto
        {
            texture_id = arrowTexture,
            x = BoxX + 20,
            y = ChoiceStartY,
            z_layer = DialogueZLayer + 2,
            scale_x = 0,
            scale_y = 0
        });

        // Start hidden
        _isVisible = false;
    }

    /// <summary>
    /// Update UI.
    /// </summary>
    public void Update(float deltaTime)
    {
        // UI is mostly event-driven, but could add animations here
    }

    /// <summary>
    /// Handle dialogue started event.
    /// </summary>
    private void OnDialogueStarted()
    {
        Show();
    }

    /// <summary>
    /// Handle dialogue ended event.
    /// </summary>
    private void OnDialogueEnded()
    {
        Hide();
    }

    /// <summary>
    /// Handle dialogue node changed event.
    /// </summary>
    private void OnNodeChanged(DialogueNode node)
    {
        // In a full implementation, we would:
        // 1. Load pre-rendered text sprite for node.Text
        // 2. Load pre-rendered sprites for each choice
        // For MVP, we just update arrow position

        // Update arrow visibility based on whether there are choices
        if (_selectionArrowSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _selectionArrowSpriteId,
                scale_x = node.HasChoices ? 0.5f : 0f,
                scale_y = node.HasChoices ? 0.5f : 0f,
                y = ChoiceStartY
            });
        }

        _game.GameLog($"[{node.SpeakerName}]: {node.Text}");

        if (node.HasChoices)
        {
            for (int i = 0; i < node.Choices.Count; i++)
            {
                _game.GameLog($"  [{i + 1}] {node.Choices[i].Text}");
            }
        }
    }

    /// <summary>
    /// Handle choice selection changed event.
    /// </summary>
    private void OnSelectionChanged(int selectedIndex)
    {
        // Move arrow to selected choice
        if (_selectionArrowSpriteId != 0)
        {
            float arrowY = ChoiceStartY + (selectedIndex * ChoiceSpacing);

            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _selectionArrowSpriteId,
                y = arrowY
            });
        }
    }

    /// <summary>
    /// Show the dialogue UI.
    /// </summary>
    public void Show()
    {
        if (_isVisible) return;
        _isVisible = true;

        if (_dialogueBoxSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _dialogueBoxSpriteId,
                scale_x = BoxWidth / 32f,
                scale_y = BoxHeight / 32f
            });
        }
    }

    /// <summary>
    /// Hide the dialogue UI.
    /// </summary>
    public void Hide()
    {
        if (!_isVisible) return;
        _isVisible = false;

        if (_dialogueBoxSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _dialogueBoxSpriteId,
                scale_x = 0,
                scale_y = 0
            });
        }

        if (_selectionArrowSpriteId != 0)
        {
            _game.UpdateSprite(new SpriteUpdateDto
            {
                id = _selectionArrowSpriteId,
                scale_x = 0,
                scale_y = 0
            });
        }
    }

    /// <summary>
    /// Clean up sprites.
    /// </summary>
    public void Destroy()
    {
        if (_dialogueBoxSpriteId != 0) _game.RemoveSprite(_dialogueBoxSpriteId);
        if (_selectionArrowSpriteId != 0) _game.RemoveSprite(_selectionArrowSpriteId);

        foreach (var spriteId in _choiceSprites)
        {
            _game.RemoveSprite(spriteId);
        }
        _choiceSprites.Clear();
    }

    public bool IsVisible => _isVisible;
}
