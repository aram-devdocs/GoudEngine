// NPCs/DialogueSystem.cs
// Simple dialogue system stub

namespace IsometricRpg.NPCs;

// Dialogue logic is simplified and moved to GameManager
// This file kept for compatibility

public static class DialogueFactory
{
    public static object CreateSampleDialogue()
    {
        return new { Text = "Hello adventurer!" };
    }
}
