import GoudEngine

let config = EngineConfig()
config.setSize(width: 800, height: 600)
     .setTitle(title: "Hello GoudEngine")

let game = config.build()
let texture = game.loadTexture(path: "assets/player.png")

while !game.shouldClose() {
    game.beginFrame(r: 0.2, g: 0.2, b: 0.2, a: 1.0)

    if game.isKeyPressed(key: .ESCAPE) {
        game.requestClose()
    }

    game.drawSprite(
        textureId: texture,
        x: 400, y: 300,
        width: 64, height: 64,
        rotation: 0,
        color: Color.white()
    )

    game.endFrame()
}
