// GameConstants.cs

public static class GameConstants
{


    // TODO: Changing TargetFps will adjust the game speed. We need to see if that is a CS issue or Rust issue
    public const uint TargetFPS = 120;
    public const uint ScreenWidth = 288;
    public const uint ScreenHeight = 512;

    public const float Gravity = 9.8f;
    public const float JumpStrength = -3.5f;
    public const float JumpCooldown = 0.33f;

    public const float PipeSpeed = 1.0f;
    public const float PipeSpawnInterval = 1.5f;
    public const int PipeWidth = 60;
    public const int PipeGap = 100;

}