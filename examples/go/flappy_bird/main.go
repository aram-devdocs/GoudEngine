// Flappy Bird Clone - GoudEngine Go Demo
//
// This is a Go port of the flappy_goud C#/Python example, demonstrating
// the Go SDK wrapper. All game logic patterns mirror the other SDK versions.
//
// Usage:
//
//	cd examples/go/flappy_bird
//	CGO_ENABLED=1 go run .
//
// Requirements:
//   - GoudEngine native library built (cargo build --release)
//   - Go 1.21+
package main

import (
	"fmt"
	"math"
	"math/rand"
	"os"
	"path/filepath"

	"github.com/aram-devdocs/goud-engine-go/goud"
)

// -- Game Constants (mirrors GameConstants.cs) --

const (
	targetFPS  = 120
	baseHeight = 112

	screenWidth  = 288
	screenHeight = 512

	gravity      = 9.8
	jumpStrength = -3.5
	jumpCooldown = 0.30

	pipeSpeed         = 1.0
	pipeSpawnInterval = 1.5
	pipeWidth         = 60
	pipeGap           = 100

	birdWidth  = 34
	birdHeight = 24

	pipeImgWidth  = 52
	pipeImgHeight = 320

	frameDuration = 0.1
	frameCount    = 3

	rotationSmoothing = 0.03
)

// Tiny embedded 8-bit PCM WAV clips (no external audio assets required).
var flapWavBytes = []byte{
	82, 73, 70, 70, 116, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0, 64, 31,
	0, 0, 64, 31, 0, 0, 1, 0, 8, 0, 100, 97, 116, 97, 80, 0, 0, 0, 127, 182, 191, 147, 87, 61, 88, 146,
	186, 177, 127, 78, 70, 108, 160, 183, 159, 110, 75, 83, 126, 168, 175, 142, 98, 79, 99, 141, 169, 162,
	127, 92, 87, 114, 150, 165, 149, 115, 92, 98, 126, 154, 158, 136, 108, 96, 109, 135, 153, 148, 126, 106,
	104, 119, 140, 148, 138, 120, 109, 112, 126, 139, 141, 131, 119, 114, 120, 130, 136, 134, 126, 121, 121,
	125, 129, 130, 128, 126, 126, 127,
}

var resetWavBytes = []byte{
	82, 73, 70, 70, 156, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0, 64, 31,
	0, 0, 64, 31, 0, 0, 1, 0, 8, 0, 100, 97, 116, 97, 120, 0, 0, 0, 127, 143, 158, 171, 181, 188, 192, 192,
	189, 182, 172, 160, 146, 131, 117, 103, 91, 81, 74, 69, 68, 70, 76, 84, 94, 105, 118, 131, 143, 154, 164,
	171, 175, 177, 176, 172, 166, 158, 148, 137, 127, 116, 106, 97, 91, 86, 84, 84, 87, 91, 98, 106, 114, 123,
	132, 141, 148, 154, 158, 161, 161, 160, 156, 152, 146, 139, 131, 124, 117, 111, 106, 102, 100, 100, 100, 103,
	106, 110, 116, 121, 126, 132, 136, 140, 143, 145, 146, 145, 144, 142, 139, 135, 131, 128, 124, 121, 119, 117,
	115, 115, 115, 116, 118, 119, 121, 123, 125, 127, 128, 130, 130, 131, 130, 130, 129, 129, 128, 127, 127, 127,
}

// -- Textures --

type textures struct {
	background uint64
	birdFrames [3]uint64
	pipe       uint64
	base       uint64
	digits     [10]uint64
}

func loadTextures(game *goud.Game, assetBase string) textures {
	t := textures{}
	fmt.Println("  Loading textures...")

	t.background = game.LoadTexture(filepath.Join(assetBase, "sprites", "background-day.png"))

	birdNames := [3]string{"bluebird-downflap.png", "bluebird-midflap.png", "bluebird-upflap.png"}
	for i, name := range birdNames {
		t.birdFrames[i] = game.LoadTexture(filepath.Join(assetBase, "sprites", name))
	}

	t.pipe = game.LoadTexture(filepath.Join(assetBase, "sprites", "pipe-green.png"))
	t.base = game.LoadTexture(filepath.Join(assetBase, "sprites", "base.png"))

	for i := 0; i < 10; i++ {
		t.digits[i] = game.LoadTexture(filepath.Join(assetBase, "sprites", fmt.Sprintf("%d.png", i)))
	}

	fmt.Println("  All textures loaded!")
	return t
}

// -- Movement --

type movement struct {
	velocity      float32
	rotation      float32
	cooldownTimer float32
}

func (m *movement) applyGravity(dt float32) {
	m.velocity += gravity * dt * targetFPS
	m.cooldownTimer = float32(math.Max(0, float64(m.cooldownTimer-dt)))
}

func (m *movement) tryJump() bool {
	if m.cooldownTimer <= 0 {
		m.velocity = jumpStrength * targetFPS
		m.cooldownTimer = jumpCooldown
		return true
	}
	return false
}

func (m *movement) updatePosition(posY, dt float32) float32 {
	newY := posY + m.velocity*dt
	targetRot := float32(math.Max(-45, math.Min(45, float64(m.velocity*3))))
	m.rotation += (targetRot - m.rotation) * rotationSmoothing
	return newY
}

func (m *movement) reset() {
	m.velocity = 0
	m.rotation = 0
	m.cooldownTimer = 0
}

// -- Bird --

type bird struct {
	x, y         float32
	move         movement
	frameIndex   int
	animTime     float32
}

func newBird() bird {
	return bird{
		x: float32(screenWidth) / 4,
		y: float32(screenHeight) / 2,
	}
}

func (b *bird) update(dt float32, jumpPressed bool, game *goud.Game) {
	b.move.applyGravity(dt)
	if jumpPressed && b.move.tryJump() {
		game.AudioPlay(flapWavBytes)
	}
	b.y = b.move.updatePosition(b.y, dt)

	b.animTime += dt
	if b.animTime >= frameDuration {
		b.frameIndex = (b.frameIndex + 1) % frameCount
		b.animTime = 0
	}
}

func (b *bird) draw(game *goud.Game, tex *textures) {
	game.DrawSprite(
		tex.birdFrames[b.frameIndex],
		b.x, b.y,
		birdWidth, birdHeight,
		b.move.rotation*math.Pi/180,
		goud.ColorWhite(),
	)
}

func (b *bird) reset() {
	b.x = float32(screenWidth) / 4
	b.y = float32(screenHeight) / 2
	b.move.reset()
	b.frameIndex = 0
	b.animTime = 0
}

func (b *bird) rect() goud.Rect {
	return goud.NewRect(b.x-birdWidth/2, b.y-birdHeight/2, birdWidth, birdHeight)
}

// -- Pipe --

type pipePair struct {
	x      float32
	gapY   float32
	scored bool
}

func newPipePair(x float32) pipePair {
	gapY := float32(pipeGap) + float32(rand.Intn(screenHeight-baseHeight-pipeGap*2))
	return pipePair{x: x, gapY: gapY}
}

func (p *pipePair) update(dt float32) {
	p.x -= pipeSpeed * dt * targetFPS
}

func (p *pipePair) draw(game *goud.Game, tex *textures) {
	topPipeY := p.gapY - float32(pipeImgHeight)
	game.DrawSprite(tex.pipe, p.x, topPipeY, pipeImgWidth, pipeImgHeight, math.Pi, goud.ColorWhite())
	bottomPipeY := p.gapY + float32(pipeGap)
	game.DrawSprite(tex.pipe, p.x, bottomPipeY, pipeImgWidth, pipeImgHeight, 0, goud.ColorWhite())
}

func (p *pipePair) collidesWithBird(birdRect goud.Rect) bool {
	topRect := goud.NewRect(p.x-pipeImgWidth/2, p.gapY-float32(pipeImgHeight), pipeImgWidth, pipeImgHeight)
	bottomRect := goud.NewRect(p.x-pipeImgWidth/2, p.gapY+float32(pipeGap), pipeImgWidth, pipeImgHeight)
	return birdRect.Intersects(topRect) || birdRect.Intersects(bottomRect)
}

// -- Game State --

type gameState int

const (
	stateMenu gameState = iota
	statePlaying
	stateGameOver
)

type flappyGame struct {
	game    *goud.Game
	tex     textures
	bird    bird
	pipes   []pipePair
	state   gameState
	score   int
	baseX   float32
	spawnTm float32
}

func newFlappyGame(game *goud.Game, assetBase string) *flappyGame {
	return &flappyGame{
		game: game,
		tex:  loadTextures(game, assetBase),
		bird: newBird(),
	}
}

func (fg *flappyGame) update(dt float32) {
	jumpPressed := fg.game.IsKeyJustPressed(goud.KeySpace) ||
		fg.game.IsMouseButtonJustPressed(goud.MouseButtonLeft)

	switch fg.state {
	case stateMenu:
		if jumpPressed {
			fg.state = statePlaying
		}
	case statePlaying:
		fg.bird.update(dt, jumpPressed, fg.game)

		// Spawn pipes
		fg.spawnTm += dt
		if fg.spawnTm >= pipeSpawnInterval {
			fg.pipes = append(fg.pipes, newPipePair(float32(screenWidth)+pipeImgWidth))
			fg.spawnTm = 0
		}

		// Update pipes
		birdR := fg.bird.rect()
		alive := fg.pipes[:0]
		for i := range fg.pipes {
			fg.pipes[i].update(dt)
			if fg.pipes[i].x < -pipeImgWidth {
				continue
			}
			if fg.pipes[i].collidesWithBird(birdR) {
				fg.gameOver()
				return
			}
			if !fg.pipes[i].scored && fg.pipes[i].x < fg.bird.x {
				fg.pipes[i].scored = true
				fg.score++
			}
			alive = append(alive, fg.pipes[i])
		}
		fg.pipes = alive

		// Ground collision
		if fg.bird.y > float32(screenHeight-baseHeight) {
			fg.gameOver()
			return
		}
		// Ceiling
		if fg.bird.y < 0 {
			fg.bird.y = 0
			fg.bird.move.velocity = 0
		}
	case stateGameOver:
		if jumpPressed {
			fg.reset()
		}
	}

	// Scroll base
	fg.baseX -= pipeSpeed * dt * targetFPS
	if fg.baseX < -float32(screenWidth) {
		fg.baseX = 0
	}
}

func (fg *flappyGame) render() {
	// Background
	fg.game.DrawSprite(fg.tex.background, 0, 0, screenWidth, screenHeight, 0, goud.ColorWhite())

	// Pipes
	for i := range fg.pipes {
		fg.pipes[i].draw(fg.game, &fg.tex)
	}

	// Base
	fg.game.DrawSprite(fg.tex.base, fg.baseX, float32(screenHeight-baseHeight), screenWidth, baseHeight, 0, goud.ColorWhite())
	fg.game.DrawSprite(fg.tex.base, fg.baseX+float32(screenWidth), float32(screenHeight-baseHeight), screenWidth, baseHeight, 0, goud.ColorWhite())

	// Bird
	fg.bird.draw(fg.game, &fg.tex)

	// Score
	fg.drawScore()

	// Overlays
	switch fg.state {
	case stateMenu:
		fg.game.DrawQuad(0, 0, screenWidth, screenHeight, goud.NewColor(0, 0, 0, 0.3))
	case stateGameOver:
		fg.game.DrawQuad(0, 0, screenWidth, screenHeight, goud.NewColor(0.8, 0, 0, 0.3))
	}
}

func (fg *flappyGame) drawScore() {
	s := fmt.Sprintf("%d", fg.score)
	digitW := float32(24)
	totalW := float32(len(s)) * digitW
	startX := (float32(screenWidth) - totalW) / 2
	for i, ch := range s {
		d := int(ch - '0')
		fg.game.DrawSprite(fg.tex.digits[d], startX+float32(i)*digitW, 30, digitW, 36, 0, goud.ColorWhite())
	}
}

func (fg *flappyGame) gameOver() {
	fg.state = stateGameOver
	fg.game.AudioPlay(resetWavBytes)
}

func (fg *flappyGame) reset() {
	fg.bird.reset()
	fg.pipes = fg.pipes[:0]
	fg.score = 0
	fg.spawnTm = 0
	fg.state = statePlaying
	fg.game.AudioPlay(flapWavBytes)
}

// -- Main --

func main() {
	// Resolve asset path (shared with C# flappy_goud)
	exePath, _ := os.Executable()
	exeDir := filepath.Dir(exePath)

	assetBase := filepath.Join(exeDir, "..", "..", "..", "examples", "csharp", "flappy_goud", "assets")
	// Try relative to cwd if the executable path doesn't work
	if _, err := os.Stat(filepath.Join(assetBase, "sprites")); err != nil {
		cwd, _ := os.Getwd()
		assetBase = filepath.Join(cwd, "..", "..", "csharp", "flappy_goud", "assets")
		if _, err := os.Stat(filepath.Join(assetBase, "sprites")); err != nil {
			// Try from repo root
			assetBase = filepath.Join(cwd, "..", "..", "..", "examples", "csharp", "flappy_goud", "assets")
		}
	}
	fmt.Printf("Asset base: %s\n", assetBase)

	game := goud.NewGame(screenWidth, screenHeight, "Flappy Bird - Go SDK")
	defer game.Destroy()

	fg := newFlappyGame(game, assetBase)

	fmt.Println("Starting game loop...")
	for !game.ShouldClose() {
		game.BeginFrame(0.39, 0.58, 0.93, 1.0)
		dt := game.DeltaTime()
		fg.update(dt)
		fg.render()
		game.EndFrame()
	}

	fmt.Println("Game closed.")
}
