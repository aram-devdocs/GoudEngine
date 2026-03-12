float speed = 200f;

while (!game.ShouldClose())
{
    game.BeginFrame(0.1f, 0.1f, 0.1f, 1.0f);

    float dt = game.DeltaTime;

    if (game.IsKeyPressed(Keys.Escape))
    {
        game.Close();
        continue;
    }

    if (game.IsKeyPressed(Keys.Left))  x -= speed * dt;
    if (game.IsKeyPressed(Keys.Right)) x += speed * dt;
    if (game.IsKeyPressed(Keys.Up))    y -= speed * dt;
    if (game.IsKeyPressed(Keys.Down))  y += speed * dt;
    if (game.IsKeyPressed(Keys.Space)) { }

    game.DrawSprite(textureId, x, y, width, height);

    game.EndFrame();
}
