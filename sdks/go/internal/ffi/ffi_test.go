package ffi

import "testing"

// These tests require libgoud_engine to be built and available in target/debug.
// Run: cargo build && cd sdks/go && go test ./internal/ffi/ -v

// TestNilPointerGuards verifies that functions with pointer parameters
// return safely when passed nil, rather than crashing via a null C dereference.
func TestNilPointerGuards(t *testing.T) {
	// GoudAnimationClipBuilderFree takes *C.FfiAnimationClipBuilder.
	// Passing nil should return immediately without panicking.
	GoudAnimationClipBuilderFree(nil)

	// GoudAnimationClipBuilderAddFrame returns *C.FfiAnimationClipBuilder.
	// Nil input should return nil output.
	result := GoudAnimationClipBuilderAddFrame(nil, 0, 0, 0, 0)
	if result != nil {
		t.Errorf("expected nil return for nil builder, got %v", result)
	}
}

// TestSnakeToPascalNaming verifies that generated function names follow
// Go PascalCase conventions (spot check — compile-time verification).
func TestSnakeToPascalNaming(t *testing.T) {
	_ = GoudAnimationClipBuilderAddFrame
	_ = GoudAnimationClipBuilderFree
	_ = GoudAnimationClipBuilderNew
}
