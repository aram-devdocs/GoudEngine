package goud

import "testing"

func TestGoudErrorFormat(t *testing.T) {
	err := &GoudError{
		Code:      100,
		Message:   "test error",
		Category:  "Resource",
		Subsystem: "textures",
		Operation: "load",
		Recovery:  RecoveryClassRecoverable,
	}

	s := err.Error()
	if s == "" {
		t.Error("expected non-empty error string")
	}
	// Check that key parts are present
	if s != "GoudError(code=100, category=Resource, recovery=recoverable): test error" {
		t.Errorf("unexpected error format: %s", s)
	}
}

func TestRecoveryClassString(t *testing.T) {
	tests := []struct {
		rc   RecoveryClass
		want string
	}{
		{RecoveryClassRecoverable, "recoverable"},
		{RecoveryClassFatal, "fatal"},
		{RecoveryClassDegraded, "degraded"},
		{RecoveryClass(99), "unknown"},
	}
	for _, tt := range tests {
		got := tt.rc.String()
		if got != tt.want {
			t.Errorf("RecoveryClass(%d).String() = %q, want %q", tt.rc, got, tt.want)
		}
	}
}

func TestCategoryFromCode(t *testing.T) {
	tests := []struct {
		code int32
		want string
	}{
		{1, "Context"},
		{100, "Resource"},
		{200, "Graphics"},
		{300, "Entity"},
		{400, "Input"},
		{500, "System"},
		{600, "Provider"},
		{900, "Internal"},
	}
	for _, tt := range tests {
		got := categoryFromCode(tt.code)
		if got != tt.want {
			t.Errorf("categoryFromCode(%d) = %q, want %q", tt.code, got, tt.want)
		}
	}
}
