using GoudEngine;

namespace IsometricRpg.NPCs;

/// <summary>
/// A single dialogue choice.
/// </summary>
public class DialogueChoice
{
    public string Text { get; set; } = "";
    public string? NextNodeId { get; set; }
    public Action? OnSelected { get; set; }

    public DialogueChoice(string text, string? nextNodeId = null, Action? onSelected = null)
    {
        Text = text;
        NextNodeId = nextNodeId;
        OnSelected = onSelected;
    }
}

/// <summary>
/// A single node in a dialogue tree.
/// </summary>
public class DialogueNode
{
    public string Id { get; set; } = "";
    public string SpeakerName { get; set; } = "";
    public string Text { get; set; } = "";
    public List<DialogueChoice> Choices { get; set; } = new();

    // For linear dialogue (no choices)
    public string? NextNodeId { get; set; }

    public bool HasChoices => Choices.Count > 0;
}

/// <summary>
/// A complete dialogue tree.
/// </summary>
public class Dialogue
{
    public string Id { get; set; } = "";
    public string StartNodeId { get; set; } = "";
    public Dictionary<string, DialogueNode> Nodes { get; set; } = new();

    public DialogueNode? GetStartNode()
    {
        return Nodes.GetValueOrDefault(StartNodeId);
    }

    public DialogueNode? GetNode(string nodeId)
    {
        return Nodes.GetValueOrDefault(nodeId);
    }
}

/// <summary>
/// Manages dialogue flow and player interaction.
/// </summary>
public class DialogueSystem
{
    private readonly GoudGame _game;

    // Current dialogue state
    private Dialogue? _currentDialogue;
    private DialogueNode? _currentNode;
    private int _selectedChoiceIndex;
    private bool _inputCooldown;
    private float _inputCooldownTimer;
    private const float InputCooldownDuration = 0.2f;

    // Events
    public event Action<DialogueNode>? OnDialogueNodeChanged;
    public event Action<int>? OnChoiceSelectionChanged;
    public event Action? OnDialogueStarted;
    public event Action? OnDialogueEnded;

    // Key codes
    private const int KeyW = 87;
    private const int KeyS = 83;
    private const int KeySpace = 32;
    private const int KeyEnter = 257;
    private const int KeyUp = 265;
    private const int KeyDown = 264;

    public DialogueSystem(GoudGame game)
    {
        _game = game;
    }

    /// <summary>
    /// Check if dialogue is currently active.
    /// </summary>
    public bool IsDialogueActive => _currentDialogue != null;

    /// <summary>
    /// Get current dialogue node.
    /// </summary>
    public DialogueNode? CurrentNode => _currentNode;

    /// <summary>
    /// Get current selected choice index.
    /// </summary>
    public int SelectedChoiceIndex => _selectedChoiceIndex;

    /// <summary>
    /// Start a dialogue.
    /// </summary>
    public void StartDialogue(Dialogue dialogue)
    {
        _currentDialogue = dialogue;
        _currentNode = dialogue.GetStartNode();
        _selectedChoiceIndex = 0;
        _inputCooldown = true;
        _inputCooldownTimer = InputCooldownDuration;

        OnDialogueStarted?.Invoke();

        if (_currentNode != null)
        {
            OnDialogueNodeChanged?.Invoke(_currentNode);
        }

        _game.GameLog($"Started dialogue: {dialogue.Id}");
    }

    /// <summary>
    /// Update dialogue system - handle input.
    /// </summary>
    public void Update(float deltaTime)
    {
        if (!IsDialogueActive || _currentNode == null) return;

        // Handle input cooldown
        if (_inputCooldown)
        {
            _inputCooldownTimer -= deltaTime;
            if (_inputCooldownTimer <= 0)
            {
                _inputCooldown = false;
            }
            return;
        }

        // Navigate choices
        if (_currentNode.HasChoices)
        {
            if (_game.IsKeyPressed(KeyW) || _game.IsKeyPressed(KeyUp))
            {
                NavigateChoice(-1);
            }
            else if (_game.IsKeyPressed(KeyS) || _game.IsKeyPressed(KeyDown))
            {
                NavigateChoice(1);
            }
        }

        // Select/advance
        if (_game.IsKeyPressed(KeySpace) || _game.IsKeyPressed(KeyEnter))
        {
            if (_currentNode.HasChoices)
            {
                SelectChoice(_selectedChoiceIndex);
            }
            else
            {
                AdvanceDialogue();
            }
        }
    }

    /// <summary>
    /// Navigate up/down through choices.
    /// </summary>
    private void NavigateChoice(int direction)
    {
        if (_currentNode == null || !_currentNode.HasChoices) return;

        int newIndex = _selectedChoiceIndex + direction;
        newIndex = Math.Clamp(newIndex, 0, _currentNode.Choices.Count - 1);

        if (newIndex != _selectedChoiceIndex)
        {
            _selectedChoiceIndex = newIndex;
            OnChoiceSelectionChanged?.Invoke(_selectedChoiceIndex);
            SetInputCooldown();
        }
    }

    /// <summary>
    /// Select the current choice.
    /// </summary>
    private void SelectChoice(int index)
    {
        if (_currentNode == null || !_currentNode.HasChoices) return;
        if (index < 0 || index >= _currentNode.Choices.Count) return;

        var choice = _currentNode.Choices[index];

        _game.GameLog($"Selected choice: {choice.Text}");

        // Execute callback if present
        choice.OnSelected?.Invoke();

        // Navigate to next node or end dialogue
        if (choice.NextNodeId == null)
        {
            EndDialogue();
        }
        else
        {
            GoToNode(choice.NextNodeId);
        }
    }

    /// <summary>
    /// Advance linear dialogue (no choices).
    /// </summary>
    private void AdvanceDialogue()
    {
        if (_currentNode == null) return;

        if (_currentNode.NextNodeId == null)
        {
            EndDialogue();
        }
        else
        {
            GoToNode(_currentNode.NextNodeId);
        }
    }

    /// <summary>
    /// Go to a specific dialogue node.
    /// </summary>
    private void GoToNode(string nodeId)
    {
        if (_currentDialogue == null) return;

        var node = _currentDialogue.GetNode(nodeId);
        if (node == null)
        {
            _game.GameLog($"Dialogue node not found: {nodeId}");
            EndDialogue();
            return;
        }

        _currentNode = node;
        _selectedChoiceIndex = 0;
        SetInputCooldown();

        OnDialogueNodeChanged?.Invoke(node);
    }

    /// <summary>
    /// End the current dialogue.
    /// </summary>
    public void EndDialogue()
    {
        _currentDialogue = null;
        _currentNode = null;
        _selectedChoiceIndex = 0;

        OnDialogueEnded?.Invoke();

        _game.GameLog("Dialogue ended");
    }

    /// <summary>
    /// Set input cooldown to prevent rapid input.
    /// </summary>
    private void SetInputCooldown()
    {
        _inputCooldown = true;
        _inputCooldownTimer = InputCooldownDuration;
    }
}

/// <summary>
/// Factory for creating sample dialogues.
/// </summary>
public static class DialogueFactory
{
    /// <summary>
    /// Create a sample NPC dialogue with choices.
    /// </summary>
    public static Dialogue CreateSampleDialogue()
    {
        var dialogue = new Dialogue
        {
            Id = "npc_greeting",
            StartNodeId = "start"
        };

        // Start node
        dialogue.Nodes["start"] = new DialogueNode
        {
            Id = "start",
            SpeakerName = "Village Elder",
            Text = "Greetings, adventurer! The village is threatened by monsters. Will you help us?",
            Choices = new List<DialogueChoice>
            {
                new("I'll help you!", "accept"),
                new("Tell me more.", "info"),
                new("Not interested.", "refuse")
            }
        };

        // Accept path
        dialogue.Nodes["accept"] = new DialogueNode
        {
            Id = "accept",
            SpeakerName = "Village Elder",
            Text = "Thank you, brave hero! Head to the arena and defeat the monsters!",
            NextNodeId = null // End dialogue
        };

        // Info path
        dialogue.Nodes["info"] = new DialogueNode
        {
            Id = "info",
            SpeakerName = "Village Elder",
            Text = "Monsters have been attacking from the east. They're dangerous, but I believe you can defeat them.",
            Choices = new List<DialogueChoice>
            {
                new("I'll do it!", "accept"),
                new("I need to prepare first.", "prepare")
            }
        };

        // Prepare path
        dialogue.Nodes["prepare"] = new DialogueNode
        {
            Id = "prepare",
            SpeakerName = "Village Elder",
            Text = "Take your time. Come back when you're ready.",
            NextNodeId = null // End dialogue
        };

        // Refuse path
        dialogue.Nodes["refuse"] = new DialogueNode
        {
            Id = "refuse",
            SpeakerName = "Village Elder",
            Text = "I understand. Perhaps another adventurer will come...",
            NextNodeId = null // End dialogue
        };

        return dialogue;
    }
}
