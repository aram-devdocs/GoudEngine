package main

import "github.com/aram-devdocs/GoudEngine/sdks/go/goud"

func main() {
    game := goud.NewGame(800, 600, "My Game")
    defer game.Destroy()

    tex := game.LoadTexture("assets/player.png")

    for !game.ShouldClose() {
        game.BeginFrame(0, 0, 0, 1)

        if game.IsKeyJustPressed(goud.KeyEscape) {
            game.Close()
        }

        game.DrawSprite(tex, 400, 300, 64, 64, 0, goud.ColorWhite())
        game.EndFrame()
    }
}
