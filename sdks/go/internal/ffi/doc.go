// Package ffi provides raw cgo bindings to the GoudEngine C API.
//
// This package is internal — use the public GoudEngine/sdks/go/goud API instead.
//
// LDFLAGS use paths relative to this source file's directory (${SRCDIR}).
// This assumes the SDK lives at sdks/go/ inside the GoudEngine repo so that
// ../../../../target/ resolves to the Cargo output directory.
package ffi

// #cgo CFLAGS: -I${SRCDIR}/../../include -Wno-deprecated-declarations
// #cgo darwin LDFLAGS: -L${SRCDIR}/../../../../target/release -L${SRCDIR}/../../../../target/debug -lgoud_engine -framework OpenGL -framework Cocoa
// #cgo linux LDFLAGS: -L${SRCDIR}/../../../../target/release -L${SRCDIR}/../../../../target/debug -lgoud_engine -lm -ldl -lpthread
// #cgo windows LDFLAGS: -L${SRCDIR}/../../../../target/release -L${SRCDIR}/../../../../target/debug -lgoud_engine -lopengl32 -lgdi32
// #include "goud_engine.h"
import "C"
