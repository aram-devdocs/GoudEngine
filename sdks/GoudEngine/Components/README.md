# GoudEngine Components

This directory contains the built-in ECS component types for the GoudEngine C# SDK.

## Overview

Components are data containers that can be attached to entities in the Entity-Component-System (ECS) architecture. Each component represents a specific aspect of game object behavior or appearance.

## Architecture

### Component Interface

All components must implement the `IComponent` interface:

```csharp
public interface IComponent
{
    ulong TypeId { get; }        // Unique type identifier hash
    int SizeInBytes { get; }     // Size for FFI compatibility
}
```

### Component Attribute

Components must be marked with the `[Component]` attribute:

```csharp
[Component(typeIdHash, sizeInBytes)]
public struct MyComponent : IComponent
{
    // Component fields...
}
```

### Type Registration

Components are automatically registered with the Rust FFI layer on first use:

```csharp
ComponentRegistry.Register<Transform2D>();  // Explicit registration
// OR
entity.AddComponent(transform);  // Auto-registration on add
```

## Built-in Components

### Transform2D

2D spatial transformation (position, rotation, scale).

```csharp
var transform = Transform2D.FromPosition(new Vector2(100f, 50f));
transform.RotateDegrees(45f);
transform.Scale = new Vector2(2f, 2f);
entity.AddComponent(transform);
```

**Size:** 20 bytes (position: 8, rotation: 4, scale: 8)

**Features:**
- Position in 2D space
- Rotation in radians (with degree helpers)
- Non-uniform scale
- Builder pattern for construction
- Direction vectors (Forward, Right)

### Sprite

2D sprite rendering component.

```csharp
var sprite = new Sprite(textureHandle: 123)
    .WithColor(Color.Red)
    .WithFlipX()
    .WithSourceRect(new Rect(0, 0, 32, 32))
    .WithAnchor(0.5f, 1.0f);
entity.AddComponent(sprite);
```

**Size:** 48 bytes (approximate, depends on padding)

**Features:**
- Texture asset handle
- RGBA color tint
- Source rectangle for sprite sheets
- Horizontal/vertical flipping
- Customizable anchor point
- Custom size override
- Builder pattern for configuration

## Usage Patterns

### Basic Usage

```csharp
using var context = new GoudContext();
var entity = context.Spawn();

// Add components
var transform = Transform2D.FromPosition(new Vector2(100f, 50f));
entity.AddComponent(transform);

// Check if entity has component
if (entity.HasComponent<Transform2D>())
{
    // Get component (returns a copy)
    var t = entity.GetComponent<Transform2D>();
    Console.WriteLine($"Position: {t.Position}");
}
```

### Method Chaining

```csharp
var entity = context.Spawn()
    .AddComponent(Transform2D.FromPosition(new Vector2(100f, 50f)))
    .AddComponent(new Sprite(textureHandle)
        .WithColor(Color.Blue)
        .WithFlipY());
```

### Updating Components

```csharp
// Get, modify, update
var transform = entity.GetComponent<Transform2D>();
transform.Translate(new Vector2(10f, 0f));
entity.UpdateComponent(transform);

// Or use TryGetComponent
if (entity.TryGetComponent<Transform2D>(out var t))
{
    t.RotateDegrees(45f);
    entity.UpdateComponent(t);
}
```

### Removing Components

```csharp
var removed = entity.RemoveComponent<Sprite>();
if (removed)
{
    Console.WriteLine("Sprite component removed");
}
```

## Creating Custom Components

To create a custom component:

1. **Define the struct** with `#[repr(C)]` in Rust
2. **Compute the TypeId hash** (use Rust's `TypeId::of::<T>()`)
3. **Create the C# struct** with matching layout
4. **Add the Component attribute** with correct TypeId and size
5. **Implement IComponent interface**

Example:

```csharp
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential)]
[Component(0x1234567890ABCDEF, 16)]  // Replace with actual hash
public struct Velocity : IComponent
{
    public float X;
    public float Y;

    public ulong TypeId => 0x1234567890ABCDEF;
    public int SizeInBytes => 16;

    public Velocity(float x, float y)
    {
        X = x;
        Y = y;
    }
}
```

## FFI Considerations

### Memory Layout

Components use `StructLayout(LayoutKind.Sequential)` to ensure predictable memory layout for FFI:

```csharp
[StructLayout(LayoutKind.Sequential)]
public struct Transform2D
{
    public Vector2 Position;  // 8 bytes
    public float Rotation;     // 4 bytes
    public Vector2 Scale;      // 8 bytes
    // Total: 20 bytes
}
```

### Type Hashing

The TypeId must match the hash computed by Rust's `TypeId::of::<T>()`. This ensures type safety across the FFI boundary.

**TODO:** Implement automatic hash computation or code generation.

### Size Validation

The `SizeInBytes` value should match the Rust struct size. Mismatches will be caught at runtime during registration.

## Performance Notes

- **Component copies:** `GetComponent<T>()` returns a copy, not a reference. Modify and call `UpdateComponent<T>()` to persist changes.
- **Batch operations:** For updating many components, consider using system-level access (future feature).
- **Memory efficiency:** Components are stored in contiguous arrays in Rust for cache-friendly iteration.

## Future Enhancements

- [ ] Automatic TypeId hash generation
- [ ] Component source generators for boilerplate reduction
- [ ] Direct mutable references (unsafe, advanced usage)
- [ ] Component reflection and editor integration
- [ ] Batch component operations
- [ ] Component change events

## See Also

- [Entity.cs](../Core/Entity.cs) - Entity wrapper with component methods
- [GoudContext.cs](../Core/GoudContext.cs) - ECS context management
- [ComponentExample.cs](../Examples/ComponentExample.cs) - Usage examples
