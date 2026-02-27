# GoudEngine Error Handling Guide

## Overview

GoudEngine uses a comprehensive error handling system that bridges Rust error codes with C# exceptions. All errors follow a consistent pattern with error code ranges, specific exception types, and detailed error information.

## Error Code Ranges

Error codes are divided into categories matching the Rust FFI layer:

| Range     | Category  | Description                        |
|-----------|-----------|-----------------------------------|
| 0         | Success   | No error                          |
| 1-99      | Context   | Context/initialization errors     |
| 100-199   | Resource  | Asset/resource loading errors     |
| 200-299   | Graphics  | Rendering/graphics errors         |
| 300-399   | Entity    | ECS entity errors                 |
| 400-499   | Input     | Input handling errors             |
| 500-599   | System    | System/platform errors            |
| 900-999   | Internal  | Internal/unexpected errors        |

## Exception Types

### Base Exception

```csharp
// Base exception for all GoudEngine errors
public class GoudEngineException : Exception
{
    public int ErrorCode { get; }
    public ErrorCategory Category { get; }
}
```

### Specific Exception Types

- **ContextException** (1-99): Context creation/destruction failures
- **ResourceException** (100-199): Asset loading failures, includes `ResourcePath` property
- **GraphicsException** (200-299): Shader compilation, texture creation failures
- **EntityException** (300-399): Entity operations, includes `EntityId` property
- **InputException** (400-499): Input device errors
- **SystemException** (500-599): Window creation, platform errors
- **InternalException** (900-999): Unexpected internal errors

## Usage Examples

### Basic Error Handling

```csharp
using GoudEngine.Core;

try
{
    var context = new GoudContext();
    var entity = context.Spawn();

    // Component operations can throw
    entity.AddComponent(new Transform2D());
}
catch (ContextException ex)
{
    // Context creation failed
    Console.WriteLine($"Context error {ex.ErrorCode}: {ex.Message}");
}
catch (EntityException ex)
{
    // Entity operation failed
    Console.WriteLine($"Entity {ex.EntityId} error: {ex.Message}");
}
catch (GoudEngineException ex)
{
    // Generic engine error
    Console.WriteLine($"Engine error [{ex.Category}]: {ex.Message}");
}
```

### Resource Loading with Path

```csharp
try
{
    var texture = assetServer.Load<Texture>("assets/player.png");
}
catch (ResourceException ex)
{
    Console.WriteLine($"Failed to load {ex.ResourcePath}: {ex.Message}");
    // ResourcePath = "assets/player.png"
}
```

### Using ErrorHelper

```csharp
using GoudEngine.Core;

// Create exception from error code
var ex = ErrorHelper.CreateException(100, "Resource not found");
// Returns ResourceException instance

// Validate FFI results
var result = NativeMethods.some_operation(contextId);
ErrorHelper.ThrowIfFailed(result, "Operation description");

// Validate IDs before passing to FFI
ErrorHelper.ValidateEntityId(entityId, nameof(entityId));
ErrorHelper.ValidateContextId(contextId, nameof(contextId));
ErrorHelper.ValidateHandle(handle, nameof(handle));
```

### Safe Execution with TryExecute

```csharp
using GoudEngine.Core;

// Try to spawn entity without throwing
int result;
Exception? error;

if (ErrorExtensions.TryExecute(() => context.SpawnEntity(), out result, out error))
{
    Console.WriteLine($"Spawned entity: {result}");
}
else
{
    Console.WriteLine($"Spawn failed: {error.Message}");
}

// Try action without result
if (ErrorExtensions.TryExecute(() => entity.Despawn(), out error))
{
    Console.WriteLine("Entity despawned successfully");
}
else
{
    Console.WriteLine($"Despawn failed: {error.Message}");
}
```

### Pattern: Try Methods

Many entity methods have both throwing and non-throwing variants:

```csharp
// Throws EntityException if entity doesn't have component
Transform2D transform = entity.GetComponent<Transform2D>();

// Returns false if entity doesn't have component
if (entity.TryGetComponent<Transform2D>(out var transform))
{
    // Component exists, use transform
}
else
{
    // Component doesn't exist
}
```

## Error Information

All exceptions include:

- **ErrorCode**: Numeric error code from Rust FFI layer
- **Category**: Error category enum (Context, Resource, etc.)
- **Message**: Human-readable error description
- **ToString()**: Formatted string with code and category: `[GOUD-100] Resource: File not found`

Specific exception types include additional fields:

- **ResourceException.ResourcePath**: Path to the failed resource
- **EntityException.EntityId**: Entity involved in the error

## Best Practices

### 1. Use Specific Catch Blocks

```csharp
try
{
    // Operations
}
catch (ResourceException ex)
{
    // Handle missing assets
    Console.WriteLine($"Asset not found: {ex.ResourcePath}");
}
catch (EntityException ex)
{
    // Handle entity errors
    Console.WriteLine($"Entity {ex.EntityId} operation failed");
}
catch (GoudEngineException ex)
{
    // Handle other engine errors
    Console.WriteLine($"Engine error: {ex}");
}
```

### 2. Validate Before FFI Calls

```csharp
public void DoOperation(GoudEntityId entityId)
{
    // Validate inputs before FFI call
    ErrorHelper.ValidateEntityId(entityId, nameof(entityId));

    // Make FFI call
    var result = NativeMethods.some_operation(entityId);

    // Check result
    ErrorHelper.ThrowIfFailed(result, "Operation name");
}
```

### 3. Use Try Methods for Optional Components

```csharp
// DON'T: Throw and catch for control flow
try
{
    var velocity = entity.GetComponent<Velocity>();
    // Use velocity
}
catch (EntityException)
{
    // Component doesn't exist
}

// DO: Use TryGetComponent
if (entity.TryGetComponent<Velocity>(out var velocity))
{
    // Use velocity
}
```

### 4. Log Error Details

```csharp
catch (GoudEngineException ex)
{
    logger.LogError(
        "Engine operation failed. " +
        "Code: {ErrorCode}, " +
        "Category: {Category}, " +
        "Message: {Message}",
        ex.ErrorCode,
        ex.Category,
        ex.Message
    );
}
```

### 5. Provide Context in Custom Exceptions

```csharp
try
{
    entity.AddComponent(component);
}
catch (GoudEngineException ex)
{
    throw new InvalidOperationException(
        $"Failed to add {typeof(T).Name} to entity {entity.Id}",
        ex
    );
}
```

## Thread Safety

All exception types are immutable and thread-safe. Error information from the FFI layer is stored in thread-local storage, ensuring no race conditions when multiple threads interact with the engine.

## Future Improvements

- **Phase 6**: Full FFI error message retrieval from `goud_last_error_message()`
- **Phase 6**: Error code constants matching Rust `GoudErrorCode` enum
- Localized error messages
- Structured error data (JSON/structured logging)
