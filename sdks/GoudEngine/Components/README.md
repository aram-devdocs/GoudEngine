# GoudEngine Components

This directory contains the built-in ECS component types for the GoudEngine C# SDK.

## Overview

Components are pure data containers that can be attached to entities in the Entity-Component-System (ECS) architecture. Each component represents a specific aspect of game object behavior or appearance.

**Architecture Principle:** All component logic lives in Rust. C# components are pure data structs with no methods beyond construction and property access. Transformation logic (translate, rotate, etc.) is implemented via Rust FFI functions.

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

Components are automatically registered with the Rust FFI layer when first added to an entity:

```csharp
entity.AddComponent(transform);  // Auto-registration on first add
```

## Built-in Components

### Transform2D

2D spatial transformation (position, rotation, scale).

```csharp
// Create via factory methods
var transform = Transform2D.FromPosition(100f, 50f);

// Or set fields directly
var transform = new Transform2D(100f, 50f, 0f, 1f, 1f);
transform.Rotation = MathF.PI / 4; // 45 degrees
transform.Scale = new Vector2(2f, 2f);

entity.AddComponent(transform);
```

**Size:** 20 bytes

| Field | Type | Description |
|-------|------|-------------|
| PositionX | f32 | X position in world space |
| PositionY | f32 | Y position in world space |
| Rotation | f32 | Rotation angle in radians |
| ScaleX | f32 | Scale along X axis |
| ScaleY | f32 | Scale along Y axis |

**Properties:**
- `Position` - Gets/sets position as Vector2
- `Scale` - Gets/sets scale as Vector2
- `RotationDegrees` - Gets rotation in degrees (read-only)

### Sprite

2D sprite rendering component.

```csharp
// Create a sprite
var sprite = new Sprite(textureHandle: 123);

// Configure by setting fields
sprite.Color = Color.Red;
sprite.FlipX = true;
sprite.AnchorX = 0.5f;
sprite.AnchorY = 1.0f;

// Source rectangle for sprite sheets
sprite.SourceRectX = 0;
sprite.SourceRectY = 0;
sprite.SourceRectWidth = 32;
sprite.SourceRectHeight = 32;
sprite.HasSourceRect = true;

entity.AddComponent(sprite);
```

**Size:** 48 bytes (approximate, depends on padding)

| Field | Type | Description |
|-------|------|-------------|
| TextureHandle | u64 | Texture asset handle |
| ColorR/G/B/A | f32 | RGBA color tint (0.0-1.0) |
| SourceRectX/Y/Width/Height | f32 | Source rectangle for sprite sheets |
| HasSourceRect | bool | Whether source rect is active |
| FlipX | bool | Horizontal flip flag |
| FlipY | bool | Vertical flip flag |
| AnchorX/Y | f32 | Anchor point (normalized 0-1) |
| CustomSizeX/Y | f32 | Custom size override |
| HasCustomSize | bool | Whether custom size is active |

**Properties:**
- `Color` - Gets/sets the color tint as Color struct
- `Anchor` - Gets/sets anchor as Vector2
- `IsFlipped` - True if flipped on either axis

## Usage Patterns

### Basic Usage

```csharp
using var context = new GoudContext();
var entity = context.Spawn();

// Add components
var transform = Transform2D.FromPosition(100f, 50f);
entity.AddComponent(transform);

// Check if entity has component
if (entity.HasComponent<Transform2D>())
{
    // Get component (returns a copy)
    var t = entity.GetComponent<Transform2D>();
    Console.WriteLine($"Position: ({t.PositionX}, {t.PositionY})");
}
```

### Method Chaining

```csharp
var sprite = new Sprite(textureHandle);
sprite.Color = Color.Blue;
sprite.FlipY = true;

var entity = context.Spawn()
    .AddComponent(Transform2D.FromPosition(100f, 50f))
    .AddComponent(sprite);
```

### Updating Components

```csharp
// Get, modify, update
var transform = entity.GetComponent<Transform2D>();
transform.PositionX += 10f;
entity.UpdateComponent(transform);

// Or use TryGetComponent
if (entity.TryGetComponent<Transform2D>(out var t))
{
    t.Rotation += MathF.PI / 4; // Add 45 degrees
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
2. **Add FFI functions** for any component-specific operations
3. **Create the C# struct** with matching memory layout
4. **Add the Component attribute** with TypeId and size
5. **Implement IComponent interface**

Example:

```csharp
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential)]
[Component(0x1234567890ABCDEF, 8)]
public struct Velocity : IComponent
{
    public float X;
    public float Y;

    public ulong TypeId => 0x1234567890ABCDEF;
    public int SizeInBytes => 8;

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
    public float PositionX;    // 4 bytes
    public float PositionY;    // 4 bytes
    public float Rotation;     // 4 bytes
    public float ScaleX;       // 4 bytes
    public float ScaleY;       // 4 bytes
    // Total: 20 bytes
}
```

### Type Hashing

The TypeId must match the hash computed by Rust's `TypeId::of::<T>()`. This ensures type safety across the FFI boundary.

### Size Validation

The `SizeInBytes` value should match the Rust struct size. Mismatches will be caught at runtime during registration.

## Rust FFI Functions

Component manipulation logic is implemented in Rust and accessible via FFI. All operations delegate to the Rust engine:

```csharp
// Transform2D operations (via NativeMethods)
NativeMethods.goud_transform2d_translate(ref transform, dx, dy);
NativeMethods.goud_transform2d_rotate(ref transform, radians);
NativeMethods.goud_transform2d_forward(ref transform);

// Sprite operations (via NativeMethods)
NativeMethods.goud_sprite_set_color(ref sprite, r, g, b, a);
NativeMethods.goud_sprite_set_flip_x(ref sprite, true);
NativeMethods.goud_sprite_set_source_rect(ref sprite, x, y, w, h);
```

See `NativeMethods.g.cs` for the complete list of available FFI functions.

## Performance Notes

- **Component copies:** `GetComponent<T>()` returns a copy, not a reference. Modify and call `UpdateComponent<T>()` to persist changes.
- **Pure data:** Components are pure data structs with no methods, ensuring minimal overhead.
- **Memory efficiency:** Components are stored in contiguous arrays in Rust for cache-friendly iteration.

## See Also

- [Entity.cs](../Core/Entity.cs) - Entity wrapper with component methods
- [GoudContext.cs](../Core/GoudContext.cs) - ECS context management
