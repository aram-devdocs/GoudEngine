// UI/UIManager.cs
// Simple UI manager (rendering moved to GameManager)

namespace IsometricRpg.UI;

public class SimpleUI
{
    private readonly GoudGame _game;
    private readonly ulong _titleTexture;
    private readonly ulong _pressStartTexture;
    private readonly ulong _healthBgTexture;
    private readonly ulong _healthFillTexture;
    private readonly ulong _dialogueBoxTexture;

    public SimpleUI(GoudGame game, ulong titleTexture, ulong pressStartTexture,
        ulong healthBgTexture, ulong healthFillTexture, ulong dialogueBoxTexture)
    {
        _game = game;
        _titleTexture = titleTexture;
        _pressStartTexture = pressStartTexture;
        _healthBgTexture = healthBgTexture;
        _healthFillTexture = healthFillTexture;
        _dialogueBoxTexture = dialogueBoxTexture;
    }

    // UI drawing is now handled directly in GameManager.Draw methods
}
