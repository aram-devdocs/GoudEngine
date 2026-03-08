# Error Handling

GoudEngine uses structured error codes with contextual diagnostics for debugging.

## Error Codes

Error codes are organized by category ranges:

| Range | Category | Examples |
|---|---|---|
| 0 | Success | `SUCCESS` |
| 1–99 | Context | `ERR_NOT_INITIALIZED`, `ERR_ALREADY_INITIALIZED` |
| 100–199 | Resource | `ERR_RESOURCE_NOT_FOUND`, `ERR_HANDLE_EXPIRED` |
| 200–299 | Graphics | `ERR_SHADER_COMPILATION_FAILED` |
| 300–399 | Entity/ECS | `ERR_ENTITY_NOT_FOUND`, `ERR_COMPONENT_NOT_FOUND` |
| 400–499 | Input | Input-related errors |
| 500–599 | System | Platform and window errors |
| 600–699 | Provider | Provider initialization and operation errors |
| 900–999 | Internal | Unexpected failures |

## GoudError

The `GoudError` enum covers all error conditions. Key variants:

- `NotInitialized` / `AlreadyInitialized` — context lifecycle errors
- `ResourceNotFound(String)` / `ResourceLoadFailed(String)` — asset system errors
- `EntityNotFound` / `ComponentNotFound` — ECS errors
- `PhysicsInitFailed(String)` / `AudioInitFailed(String)` — provider errors
- `InternalError(String)` — unexpected internal failures

All errors convert to `i32` codes for FFI. Detailed messages are stored in thread-local storage.

## FFI Error Retrieval

SDK code retrieves error details after a failed FFI call:

- `goud_get_last_error()` — returns the `i32` error code
- `goud_get_last_error_message()` — returns a human-readable message
- `goud_last_error_subsystem()` — which subsystem produced the error
- `goud_last_error_operation()` — which operation failed
- `goud_last_error_backtrace()` — stack trace (when diagnostic mode is enabled)
- `goud_clear_last_error()` — reset error state

## Error Context

Each error records:

- **Subsystem**: which engine module produced the error (e.g., `"physics"`, `"audio"`)
- **Operation**: what was being attempted (e.g., `"create_body"`, `"load_texture"`)
- **Severity**: `Fatal`, `Recoverable`, or `Warning`

## Recovery

Errors include a `RecoveryClass`:

- `Fatal` — unrecoverable; the engine must shut down
- `Recoverable` — caller can handle and continue
- `Retry` — transient failure; retrying may succeed

## Diagnostic Mode

When enabled, errors capture backtraces at the point of creation. This is useful for debugging but adds overhead. Diagnostic mode is disabled by default in release builds.

## Source

Error system implementation is in `goud_engine/src/core/error/`:

- `types.rs` — `GoudError` enum
- `codes.rs` — error code constants
- `diagnostic.rs` — backtrace capture
- `recovery.rs` — recovery strategies
- `ffi_bridge.rs` — thread-local FFI error state
