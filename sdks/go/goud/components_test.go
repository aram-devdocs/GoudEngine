package goud

import "testing"

func TestNewTransform2DDefaults(t *testing.T) {
	tr := NewTransform2D()
	if tr.PositionX != 0 || tr.PositionY != 0 {
		t.Errorf("expected position (0,0), got (%v,%v)", tr.PositionX, tr.PositionY)
	}
	if tr.Rotation != 0 {
		t.Errorf("expected rotation 0, got %v", tr.Rotation)
	}
	if tr.ScaleX != 1.0 || tr.ScaleY != 1.0 {
		t.Errorf("expected scale (1,1), got (%v,%v)", tr.ScaleX, tr.ScaleY)
	}
}

func TestNewSpriteDefaults(t *testing.T) {
	s := NewSprite()
	if s.TextureHandle != 0 {
		t.Errorf("expected texture handle 0, got %v", s.TextureHandle)
	}
	if s.HasSourceRect {
		t.Error("expected HasSourceRect false")
	}
	if s.FlipX || s.FlipY {
		t.Error("expected no flips")
	}
}
