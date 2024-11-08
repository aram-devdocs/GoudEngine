// GameConstants.cs

public static class GameConstants
{


    // TODO: Changing TargetFps will adjust the game speed. We need to see if that is a CS issue or Rust issue
    public const uint TargetFPS = 120;
    public const uint ScreenWidth = 288;
    public const uint ScreenHeight = 512;

    public const float Gravity = 9.8f;
    public const float JumpStrength = -2f;
    public const float PipeSpeed = 0.8f;
    public const float PipeSpawnInterval = 4f;
    public const int BirdWidth = 34;
    public const int BirdHeight = 24;
    public const int PipeWidth = 60;
    public const int PipeGap = 100;
}