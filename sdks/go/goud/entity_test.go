package goud

import "testing"

func TestEntityIDIndexGeneration(t *testing.T) {
	e := NewEntityID(0x0000000200000001)
	if e.Index() != 1 {
		t.Errorf("expected index 1, got %d", e.Index())
	}
	if e.Generation() != 2 {
		t.Errorf("expected generation 2, got %d", e.Generation())
	}
}

func TestEntityIDPlaceholder(t *testing.T) {
	e := NewEntityID(0xFFFFFFFFFFFFFFFF)
	if !e.IsPlaceholder() {
		t.Error("expected placeholder to be true")
	}
}

func TestEntityIDNotPlaceholder(t *testing.T) {
	e := NewEntityID(42)
	if e.IsPlaceholder() {
		t.Error("expected placeholder to be false for regular entity")
	}
}

func TestEntityIDBits(t *testing.T) {
	e := NewEntityID(12345)
	if e.Bits() != 12345 {
		t.Errorf("expected bits 12345, got %d", e.Bits())
	}
}

func TestEntityIDString(t *testing.T) {
	e := NewEntityID(0x0000000300000005)
	s := e.String()
	expected := "Entity(5v3)"
	if s != expected {
		t.Errorf("expected %q, got %q", expected, s)
	}
}
