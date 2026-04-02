using GoudEngine;

using var config = new EngineConfig()
    .SetSize(800, 600)
    .SetTitle("My Game");
using var game = config.Build();

var tex = game.LoadTexture("assets/player.png");

while (!game.ShouldClose())
{
    game.BeginFrame(0, 0, 0, 1);
    if (game.IsKeyPressed(Key.Escape)) { break; }
    game.DrawSprite(tex, 400, 300, 64, 64);
    game.EndFrame();
}
