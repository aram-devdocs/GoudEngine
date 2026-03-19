package goud

import (
	"math"
	"testing"
)

func approxEq(a, b, epsilon float32) bool {
	return float32(math.Abs(float64(a-b))) < epsilon
}

func TestColorWhite(t *testing.T) {
	c := ColorWhite()
	if c.R != 1 || c.G != 1 || c.B != 1 || c.A != 1 {
		t.Errorf("expected white (1,1,1,1), got (%v,%v,%v,%v)", c.R, c.G, c.B, c.A)
	}
}

func TestColorBlack(t *testing.T) {
	c := ColorBlack()
	if c.R != 0 || c.G != 0 || c.B != 0 || c.A != 1 {
		t.Errorf("expected black (0,0,0,1), got (%v,%v,%v,%v)", c.R, c.G, c.B, c.A)
	}
}

func TestColorFromHex(t *testing.T) {
	c := ColorFromHex(0xFF8000)
	if !approxEq(c.R, 1.0, 0.01) || !approxEq(c.G, 0.502, 0.01) || !approxEq(c.B, 0.0, 0.01) {
		t.Errorf("fromHex(0xFF8000) = (%v,%v,%v), expected ~(1.0, 0.502, 0.0)", c.R, c.G, c.B)
	}
}

func TestColorWithAlpha(t *testing.T) {
	c := ColorWhite().WithAlpha(0.5)
	if c.A != 0.5 {
		t.Errorf("expected alpha 0.5, got %v", c.A)
	}
	if c.R != 1 || c.G != 1 || c.B != 1 {
		t.Errorf("WithAlpha should preserve RGB, got (%v,%v,%v)", c.R, c.G, c.B)
	}
}

func TestColorLerp(t *testing.T) {
	a := ColorBlack()
	b := ColorWhite()
	c := a.Lerp(b, 0.5)
	if !approxEq(c.R, 0.5, 0.01) || !approxEq(c.G, 0.5, 0.01) || !approxEq(c.B, 0.5, 0.01) {
		t.Errorf("lerp(black, white, 0.5) = (%v,%v,%v), expected ~(0.5,0.5,0.5)", c.R, c.G, c.B)
	}
}

func TestVec2Zero(t *testing.T) {
	v := Vec2Zero()
	if v.X != 0 || v.Y != 0 {
		t.Errorf("expected zero (0,0), got (%v,%v)", v.X, v.Y)
	}
}

func TestVec2Add(t *testing.T) {
	a := NewVec2(1, 2)
	b := NewVec2(3, 4)
	c := a.Add(b)
	if c.X != 4 || c.Y != 6 {
		t.Errorf("expected (4,6), got (%v,%v)", c.X, c.Y)
	}
}

func TestVec2Sub(t *testing.T) {
	a := NewVec2(5, 7)
	b := NewVec2(2, 3)
	c := a.Sub(b)
	if c.X != 3 || c.Y != 4 {
		t.Errorf("expected (3,4), got (%v,%v)", c.X, c.Y)
	}
}

func TestVec2Scale(t *testing.T) {
	v := NewVec2(2, 3)
	s := v.Scale(2)
	if s.X != 4 || s.Y != 6 {
		t.Errorf("expected (4,6), got (%v,%v)", s.X, s.Y)
	}
}

func TestVec2Length(t *testing.T) {
	v := NewVec2(3, 4)
	l := v.Length()
	if !approxEq(l, 5, 0.001) {
		t.Errorf("expected length 5, got %v", l)
	}
}

func TestVec2Normalize(t *testing.T) {
	v := NewVec2(3, 4)
	n := v.Normalize()
	if !approxEq(n.Length(), 1.0, 0.001) {
		t.Errorf("expected unit length, got %v", n.Length())
	}
}

func TestVec2Dot(t *testing.T) {
	a := NewVec2(1, 0)
	b := NewVec2(0, 1)
	d := a.Dot(b)
	if d != 0 {
		t.Errorf("expected dot product 0, got %v", d)
	}

	c := NewVec2(2, 3)
	e := NewVec2(4, 5)
	if c.Dot(e) != 23 {
		t.Errorf("expected dot product 23, got %v", c.Dot(e))
	}
}

func TestVec2Distance(t *testing.T) {
	a := NewVec2(0, 0)
	b := NewVec2(3, 4)
	d := a.Distance(b)
	if !approxEq(d, 5, 0.001) {
		t.Errorf("expected distance 5, got %v", d)
	}
}

func TestVec2Lerp(t *testing.T) {
	a := NewVec2(0, 0)
	b := NewVec2(10, 20)
	c := a.Lerp(b, 0.5)
	if c.X != 5 || c.Y != 10 {
		t.Errorf("expected (5,10), got (%v,%v)", c.X, c.Y)
	}
}

func TestRectContains(t *testing.T) {
	r := NewRect(0, 0, 10, 10)
	if !r.Contains(NewVec2(5, 5)) {
		t.Error("expected point (5,5) inside rect")
	}
	if r.Contains(NewVec2(15, 5)) {
		t.Error("expected point (15,5) outside rect")
	}
}

func TestRectIntersects(t *testing.T) {
	a := NewRect(0, 0, 10, 10)
	b := NewRect(5, 5, 10, 10)
	c := NewRect(20, 20, 5, 5)

	if !a.Intersects(b) {
		t.Error("expected a and b to intersect")
	}
	if a.Intersects(c) {
		t.Error("expected a and c not to intersect")
	}
}
