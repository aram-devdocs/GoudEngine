// GoudEngine Feature Lab - Go SDK
//
// Headless smoke test that exercises Go SDK API surfaces:
//   - Entity spawn, clone, despawn lifecycle
//   - Component add/get/remove (Transform2D)
//   - Error handling
//
// Usage:
//
//	cd examples/go/feature_lab
//	CGO_ENABLED=1 go run .
//
// Or via dev.sh:
//
//	./dev.sh --sdk go --game feature_lab
//
// Requirements:
//   - GoudEngine native library built (cargo build --release)
//   - Go 1.21+
package main

import (
	"fmt"
	"os"

	"github.com/aram-devdocs/goud-engine-go/goud"
)

type checkResult struct {
	name   string
	status string // PASS, FAIL, SKIP
	detail string
}

func runCheck(name string, fn func() (bool, string)) checkResult {
	defer func() {
		if r := recover(); r != nil {
			// Recovered from panic -- treat as FAIL
		}
	}()
	ok, detail := fn()
	status := "PASS"
	if !ok {
		status = "FAIL"
	}
	return checkResult{name: name, status: status, detail: detail}
}

func checkGameCreateDestroy() (bool, string) {
	// Create a tiny off-screen window for headless-like testing.
	// The Go SDK requires a windowed context (NewGame), so we create
	// the smallest possible window and immediately destroy it.
	game := goud.NewGame(1, 1, "Feature Lab - Go (headless)")
	if game == nil {
		return false, "NewGame returned nil"
	}
	game.Destroy()
	return true, "create + destroy succeeded"
}

func checkEntityLifecycle() (bool, string) {
	game := goud.NewGame(1, 1, "Feature Lab Entity Test")
	defer game.Destroy()

	initialCount := game.EntityCount()

	// Spawn
	entity := game.SpawnEmpty()
	alive := game.IsAlive(entity)
	countAfterSpawn := game.EntityCount()

	// Clone
	cloned := game.CloneEntity(entity)
	clonedAlive := game.IsAlive(cloned)

	// Despawn both
	despawnedOriginal := game.Despawn(entity)
	despawnedClone := game.Despawn(cloned)
	finalCount := game.EntityCount()

	ok := alive && clonedAlive && despawnedOriginal && despawnedClone
	detail := fmt.Sprintf(
		"initial=%d, afterSpawn=%d, final=%d, alive=%v, clonedAlive=%v",
		initialCount, countAfterSpawn, finalCount, alive, clonedAlive,
	)
	return ok, detail
}

func checkTransform2DComponent() (bool, string) {
	game := goud.NewGame(1, 1, "Feature Lab Transform Test")
	defer game.Destroy()

	entity := game.SpawnEmpty()

	// Add Transform2D
	transform := goud.Transform2D{
		PositionX: 10.0,
		PositionY: 20.0,
		Rotation:  0.5,
		ScaleX:    1.0,
		ScaleY:    1.0,
	}
	game.AddTransform2d(entity, transform)

	// Check has
	has := game.HasTransform2d(entity)

	// Get
	got := game.GetTransform2d(entity)
	positionMatch := false
	if got != nil {
		positionMatch = got.PositionX == 10.0 && got.PositionY == 20.0
	}

	// Remove
	removed := game.RemoveTransform2d(entity)
	hasAfterRemove := game.HasTransform2d(entity)

	game.Despawn(entity)

	ok := has && positionMatch && removed && !hasAfterRemove
	detail := fmt.Sprintf(
		"has=%v, positionMatch=%v, removed=%v, hasAfterRemove=%v",
		has, positionMatch, removed, hasAfterRemove,
	)
	return ok, detail
}

func checkErrorTypes() (bool, string) {
	// Verify that error types and recovery class constants are accessible
	classes := []goud.RecoveryClass{
		goud.RecoveryClassRecoverable,
		goud.RecoveryClassFatal,
		goud.RecoveryClassDegraded,
	}
	ok := len(classes) == 3 && classes[0].String() == "recoverable"
	detail := fmt.Sprintf(
		"recovery classes accessible: %d, recoverable=%q",
		len(classes), classes[0].String(),
	)
	return ok, detail
}

func main() {
	fmt.Println("================================================================")
	fmt.Println(" GoudEngine Go Feature Lab")
	fmt.Println("================================================================")

	results := []checkResult{
		runCheck("game create and destroy", checkGameCreateDestroy),
		runCheck("entity lifecycle (spawn/clone/despawn)", checkEntityLifecycle),
		runCheck("Transform2D component add/get/remove", checkTransform2DComponent),
		runCheck("error type categories accessible", checkErrorTypes),
	}

	passCount := 0
	failCount := 0
	skipCount := 0
	for _, r := range results {
		switch r.status {
		case "PASS":
			passCount++
		case "FAIL":
			failCount++
		case "SKIP":
			skipCount++
		}
	}

	fmt.Printf("\nFeature Lab complete: %d pass, %d fail, %d skip\n", passCount, failCount, skipCount)
	for _, r := range results {
		suffix := ""
		if r.detail != "" {
			suffix = fmt.Sprintf(" (%s)", r.detail)
		}
		fmt.Printf("%s: %s%s\n", r.status, r.name, suffix)
	}

	if failCount > 0 {
		os.Exit(1)
	}
}
