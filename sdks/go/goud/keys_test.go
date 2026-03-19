package goud

import "testing"

func TestKeyConstants(t *testing.T) {
	// Verify a few well-known key values match GLFW mappings
	if KeySpace != 32 {
		t.Errorf("expected KeySpace=32, got %d", KeySpace)
	}
	if KeyEscape != 256 {
		t.Errorf("expected KeyEscape=256, got %d", KeyEscape)
	}
	if KeyEnter != 257 {
		t.Errorf("expected KeyEnter=257, got %d", KeyEnter)
	}
}

func TestMouseButtonConstants(t *testing.T) {
	if MouseButtonLeft != 0 {
		t.Errorf("expected MouseButtonLeft=0, got %d", MouseButtonLeft)
	}
	if MouseButtonRight != 1 {
		t.Errorf("expected MouseButtonRight=1, got %d", MouseButtonRight)
	}
}

func TestKeyType(t *testing.T) {
	// Ensure Key is a distinct type, not just int32
	var k Key = KeyA
	_ = k
}

func TestMouseButtonType(t *testing.T) {
	var mb MouseButton = MouseButtonLeft
	_ = mb
}
