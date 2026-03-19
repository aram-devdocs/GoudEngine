// Package ffi provides raw cgo bindings to the GoudEngine C API.
//
// This package is internal — use the public goud-engine-go API instead.
package ffi

// #cgo CFLAGS: -I${SRCDIR}/../../include -Wno-deprecated-declarations
// #cgo darwin LDFLAGS: -L${SRCDIR}/../../../../target/release -L${SRCDIR}/../../../../target/debug -lgoud_engine -framework OpenGL -framework Cocoa
// #cgo linux LDFLAGS: -L${SRCDIR}/../../../../target/release -L${SRCDIR}/../../../../target/debug -lgoud_engine -lm -ldl -lpthread
// #include "goud_engine.h"
import "C"
