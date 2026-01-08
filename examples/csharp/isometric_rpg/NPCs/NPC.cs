// NPCs/NPC.cs
// Simple NPC entity

using IsometricRpg.Core;

namespace IsometricRpg.NPCs;

public class SimpleNPC : SimpleEntity
{
    public string Name { get; }
    private const float InteractionRadius = 64f;

    public SimpleNPC(float x, float y, string name) : base(x, y)
    {
        Name = name;
        Width = 32;
        Height = 32;
    }

    public override void Update(float deltaTime)
    {
        // NPCs don't move in this simple version
    }

    public bool IsNearPlayer(float playerX, float playerY)
    {
        float dx = X - playerX;
        float dy = Y - playerY;
        float dist = (float)System.Math.Sqrt(dx * dx + dy * dy);
        return dist < InteractionRadius;
    }
}
