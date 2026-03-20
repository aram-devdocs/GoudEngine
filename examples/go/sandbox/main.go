// GoudEngine Sandbox - Go SDK
//
// A simplified interactive sandbox demonstrating core engine features:
//   - Window creation and configuration
//   - Texture loading and sprite drawing
//   - Colored quad drawing
//   - Keyboard and mouse input
//   - Basic game loop with mode switching
//
// Usage:
//
//	cd examples/go/sandbox
//	CGO_ENABLED=1 go run .
//
// Or via dev.sh:
//
//	./dev.sh --sdk go --game sandbox
//
// Requirements:
//   - GoudEngine native library built (cargo build --release)
//   - Go 1.21+
package main

import (
	"fmt"
	"math"
	"os"
	"path/filepath"

	"github.com/aram-devdocs/goud-engine-go/goud"
)

const (
	windowWidth  = 1280
	windowHeight = 720
	moveSpeed    = 220.0
)

// mode tracks which rendering mode the sandbox is in.
type mode int

const (
	mode2D mode = iota
	mode3D
	modeHybrid
)

func (m mode) String() string {
	switch m {
	case mode2D:
		return "2D"
	case mode3D:
		return "3D"
	case modeHybrid:
		return "Hybrid"
	default:
		return "Unknown"
	}
}

// resolveAssetBase tries several paths to locate the shared flappy_goud assets
// directory (used for background and sprite textures).
func resolveAssetBase() string {
	candidates := []string{}

	// From the example directory (when run via go run .)
	cwd, _ := os.Getwd()
	candidates = append(candidates,
		filepath.Join(cwd, "..", "..", "csharp", "flappy_goud", "assets"),
		filepath.Join(cwd, "..", "..", "..", "examples", "csharp", "flappy_goud", "assets"),
	)

	// From executable location
	exePath, _ := os.Executable()
	exeDir := filepath.Dir(exePath)
	candidates = append(candidates,
		filepath.Join(exeDir, "..", "..", "..", "examples", "csharp", "flappy_goud", "assets"),
	)

	for _, base := range candidates {
		if _, err := os.Stat(filepath.Join(base, "sprites")); err == nil {
			return base
		}
	}

	// Fallback
	return filepath.Join(cwd, "..", "..", "csharp", "flappy_goud", "assets")
}

func main() {
	assetBase := resolveAssetBase()
	fmt.Printf("Asset base: %s\n", assetBase)

	game := goud.NewGame(windowWidth, windowHeight, "GoudEngine Sandbox - Go")
	defer game.Destroy()

	// Load textures
	background := game.LoadTexture(filepath.Join(assetBase, "sprites", "background-day.png"))
	sprite := game.LoadTexture(filepath.Join(assetBase, "sprites", "bluebird-midflap.png"))
	fmt.Println("Textures loaded.")

	// State
	playerX := float32(250.0)
	playerY := float32(300.0)
	angle := float32(0.0)
	currentMode := mode2D

	fmt.Println("Starting sandbox...")
	fmt.Println("  WASD/Arrows: move sprite")
	fmt.Println("  1/2/3: switch mode (2D / 3D / Hybrid)")
	fmt.Println("  Escape: quit")

	for !game.ShouldClose() {
		game.BeginFrame(0.07, 0.10, 0.14, 1.0)
		dt := game.DeltaTime()
		angle += dt

		// Input: quit
		if game.IsKeyJustPressed(goud.KeyEscape) {
			game.Close()
		}

		// Input: mode switching
		if game.IsKeyJustPressed(goud.KeyDigit1) {
			currentMode = mode2D
			fmt.Println("Mode: 2D")
		}
		if game.IsKeyJustPressed(goud.KeyDigit2) {
			currentMode = mode3D
			fmt.Println("Mode: 3D")
		}
		if game.IsKeyJustPressed(goud.KeyDigit3) {
			currentMode = modeHybrid
			fmt.Println("Mode: Hybrid")
		}

		// Input: movement
		if game.IsKeyPressed(goud.KeyA) || game.IsKeyPressed(goud.KeyLeft) {
			playerX -= moveSpeed * dt
		}
		if game.IsKeyPressed(goud.KeyD) || game.IsKeyPressed(goud.KeyRight) {
			playerX += moveSpeed * dt
		}
		if game.IsKeyPressed(goud.KeyW) || game.IsKeyPressed(goud.KeyUp) {
			playerY -= moveSpeed * dt
		}
		if game.IsKeyPressed(goud.KeyS) || game.IsKeyPressed(goud.KeyDown) {
			playerY += moveSpeed * dt
		}

		// Rendering
		switch currentMode {
		case mode2D:
			// Full background
			game.DrawSprite(background, windowWidth/2, windowHeight/2, windowWidth, windowHeight, 0, goud.ColorWhite())
			// Movable sprite
			game.DrawSprite(sprite, playerX, playerY, 64, 64, angle*0.25, goud.ColorWhite())
			// Colored quad
			game.DrawQuad(920, 260, 180, 40, goud.NewColor(0.20, 0.55, 0.95, 0.80))

		case mode3D:
			// Placeholder: 3D mode shows a dark background with info quad
			game.DrawQuad(windowWidth/2, windowHeight/2, windowWidth, windowHeight, goud.NewColor(0.05, 0.08, 0.12, 1.0))
			game.DrawQuad(windowWidth/2, windowHeight/2-40, 400, 60, goud.NewColor(0.20, 0.55, 0.95, 0.60))

		case modeHybrid:
			// Mix: dark overlay + sprite
			game.DrawQuad(windowWidth/2, windowHeight/2, windowWidth, windowHeight, goud.NewColor(0.08, 0.17, 0.24, 1.0))
			game.DrawSprite(sprite, playerX, playerY, 72, 72, angle*0.25, goud.ColorWhite())
			game.DrawQuad(920, 260, 180, 40, goud.NewColor(0.20, 0.55, 0.95, 0.62))
		}

		// Mouse marker
		mouse := game.GetMousePosition()
		game.DrawQuad(mouse.X, mouse.Y, 14, 14, goud.NewColor(0.95, 0.85, 0.20, 0.95))

		// Mode indicator badge
		game.DrawQuad(windowWidth/2, 20, 200, 30, goud.NewColor(0.20, 0.55, 0.95, 0.84))

		// Oscillating decoration
		bobY := float32(600 + 20*math.Sin(float64(angle*2)))
		game.DrawQuad(100, bobY, 60, 60, goud.NewColor(0.90, 0.30, 0.40, 0.75))

		game.EndFrame()
	}

	fmt.Println("Sandbox closed.")
}
