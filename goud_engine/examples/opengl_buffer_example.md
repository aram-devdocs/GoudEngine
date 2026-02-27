# OpenGL Buffer Operations Example

This example demonstrates how to use the OpenGL backend's buffer management system for vertex data, index data, and uniform buffers.

## Overview

The OpenGL backend provides type-safe buffer operations with:
- **Generational handles** - Prevents use-after-free and stale handle access
- **Automatic resource cleanup** - Proper OpenGL object lifecycle management
- **Type-safe bindings** - Separate buffers for vertices, indices, and uniforms
- **Error handling** - All operations return Results for graceful error handling

## Basic Buffer Creation

```rust
use goud_engine::libs::graphics::backend::{
    OpenGLBackend, RenderBackend, BufferType, BufferUsage
};

// Create the OpenGL backend (requires active OpenGL context)
let mut backend = OpenGLBackend::new()?;

// Define vertex data (position + color)
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

let vertices = vec![
    Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0, 1.0] },
    Vertex { position: [ 0.0,  0.5, 0.0], color: [0.0, 0.0, 1.0, 1.0] },
];

// Convert to bytes
let vertex_data: &[u8] = bytemuck::cast_slice(&vertices);

// Create vertex buffer
let vertex_buffer = backend.create_buffer(
    BufferType::Vertex,
    BufferUsage::Static,  // Data won't change
    vertex_data,
)?;

println!("Created vertex buffer with {} bytes", backend.buffer_size(vertex_buffer).unwrap());
```

## Index Buffer Creation

```rust
// Create index buffer for indexed drawing
let indices: Vec<u16> = vec![0, 1, 2];
let index_data: &[u8] = bytemuck::cast_slice(&indices);

let index_buffer = backend.create_buffer(
    BufferType::Index,
    BufferUsage::Static,
    index_data,
)?;

// Bind both buffers for rendering
backend.bind_buffer(vertex_buffer)?;
backend.bind_buffer(index_buffer)?;
```

## Dynamic Buffer Updates

```rust
// Create a dynamic buffer for frequently changing data
let mut positions: Vec<[f32; 3]> = vec![
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
];

let dynamic_buffer = backend.create_buffer(
    BufferType::Vertex,
    BufferUsage::Dynamic,  // Will be updated frequently
    bytemuck::cast_slice(&positions),
)?;

// Later, update the buffer with new positions
positions[0] = [0.1, 0.1, 0.0];
backend.update_buffer(
    dynamic_buffer,
    0,  // Offset in bytes
    bytemuck::cast_slice(&positions),
)?;
```

## Uniform Buffer Example

```rust
// Create a uniform buffer for shader constants
#[repr(C)]
struct MaterialUniforms {
    color: [f32; 4],
    metallic: f32,
    roughness: f32,
    _padding: [f32; 2],  // Align to 16 bytes
}

let uniforms = MaterialUniforms {
    color: [1.0, 0.5, 0.3, 1.0],
    metallic: 0.8,
    roughness: 0.2,
    _padding: [0.0, 0.0],
};

let uniform_buffer = backend.create_buffer(
    BufferType::Uniform,
    BufferUsage::Dynamic,
    bytemuck::cast_slice(&[uniforms]),
)?;

// Update uniforms per-frame
let new_color = [0.9, 0.4, 0.2, 1.0];
backend.update_buffer(
    uniform_buffer,
    0,  // Offset to color field
    bytemuck::cast_slice(&[new_color]),
)?;
```

## Buffer Lifecycle Management

```rust
// Check if a buffer is valid
if backend.is_buffer_valid(vertex_buffer) {
    println!("Buffer is valid");
}

// Get buffer size
if let Some(size) = backend.buffer_size(vertex_buffer) {
    println!("Buffer size: {} bytes", size);
}

// Destroy buffer when done
if backend.destroy_buffer(vertex_buffer) {
    println!("Buffer destroyed successfully");
}

// Using a destroyed handle returns an error
match backend.bind_buffer(vertex_buffer) {
    Ok(_) => println!("Bound buffer"),
    Err(e) => println!("Error: {:?}", e),  // InvalidHandle error
}
```

## Handle Reuse and Generations

```rust
// Create a buffer
let handle1 = backend.create_buffer(
    BufferType::Vertex,
    BufferUsage::Static,
    &[1, 2, 3, 4],
)?;

println!("Handle 1: index={}, generation={}",
    handle1.index(), handle1.generation());

// Destroy it
backend.destroy_buffer(handle1);

// Create another buffer - reuses the slot
let handle2 = backend.create_buffer(
    BufferType::Vertex,
    BufferUsage::Static,
    &[5, 6, 7, 8],
)?;

println!("Handle 2: index={}, generation={}",
    handle2.index(), handle2.generation());

// Same index, different generation
assert_eq!(handle1.index(), handle2.index());
assert_ne!(handle1.generation(), handle2.generation());

// Old handle is invalid
assert!(!backend.is_buffer_valid(handle1));
assert!(backend.is_buffer_valid(handle2));
```

## Multiple Buffer Management

```rust
// Create multiple buffers of different types
let buffers = vec![
    backend.create_buffer(BufferType::Vertex, BufferUsage::Static, &[1, 2, 3, 4])?,
    backend.create_buffer(BufferType::Vertex, BufferUsage::Static, &[5, 6, 7, 8])?,
    backend.create_buffer(BufferType::Index, BufferUsage::Static, &[0, 1, 2])?,
];

// Process all buffers
for handle in &buffers {
    if let Some(size) = backend.buffer_size(*handle) {
        println!("Buffer {:?} has size {} bytes", handle, size);
    }
}

// Cleanup
for handle in buffers {
    backend.destroy_buffer(handle);
}
```

## Error Handling

```rust
use goud_engine::core::error::GoudError;

// Handle buffer creation failures
match backend.create_buffer(BufferType::Vertex, BufferUsage::Static, &data) {
    Ok(handle) => println!("Created buffer: {:?}", handle),
    Err(GoudError::BufferCreationFailed(msg)) => {
        eprintln!("Failed to create buffer: {}", msg);
    }
    Err(e) => eprintln!("Unexpected error: {:?}", e),
}

// Handle out-of-bounds updates
let result = backend.update_buffer(handle, 1000, &[1, 2, 3, 4]);
match result {
    Err(GoudError::InvalidState(msg)) => {
        eprintln!("Update failed: {}", msg);
    }
    _ => {}
}

// Handle invalid handle access
match backend.bind_buffer(invalid_handle) {
    Err(GoudError::InvalidHandle) => {
        eprintln!("Cannot bind invalid handle");
    }
    _ => {}
}
```

## Best Practices

### 1. Choose the Right Usage Hint

```rust
// Static: Set once, use many times (most geometry)
let static_buffer = backend.create_buffer(
    BufferType::Vertex,
    BufferUsage::Static,
    &vertices,
)?;

// Dynamic: Update frequently (particle systems, UI)
let dynamic_buffer = backend.create_buffer(
    BufferType::Vertex,
    BufferUsage::Dynamic,
    &particles,
)?;

// Stream: Set once per frame, use a few times (temporary data)
let stream_buffer = backend.create_buffer(
    BufferType::Vertex,
    BufferUsage::Stream,
    &temp_data,
)?;
```

### 2. Minimize Buffer Bindings

```rust
// Bad: Bind buffer every draw call
for mesh in meshes {
    backend.bind_buffer(mesh.vertex_buffer)?;
    backend.bind_buffer(mesh.index_buffer)?;
    // draw...
}

// Good: Sort by buffer to minimize state changes
meshes.sort_by_key(|m| (m.vertex_buffer, m.index_buffer));
let mut current_vb = None;
let mut current_ib = None;

for mesh in meshes {
    if current_vb != Some(mesh.vertex_buffer) {
        backend.bind_buffer(mesh.vertex_buffer)?;
        current_vb = Some(mesh.vertex_buffer);
    }
    if current_ib != Some(mesh.index_buffer) {
        backend.bind_buffer(mesh.index_buffer)?;
        current_ib = Some(mesh.index_buffer);
    }
    // draw...
}
```

### 3. Batch Updates

```rust
// Bad: Multiple small updates
for i in 0..100 {
    backend.update_buffer(buffer, i * 4, &[i as u8])?;
}

// Good: Single large update
let data: Vec<u8> = (0..100).map(|i| i as u8).collect();
backend.update_buffer(buffer, 0, &data)?;
```

### 4. Proper Resource Cleanup

```rust
struct Mesh {
    vertex_buffer: BufferHandle,
    index_buffer: BufferHandle,
}

impl Mesh {
    fn destroy(&self, backend: &mut OpenGLBackend) {
        backend.destroy_buffer(self.vertex_buffer);
        backend.destroy_buffer(self.index_buffer);
    }
}
```

## Performance Tips

1. **Use Static buffers when possible** - Better GPU cache locality
2. **Minimize buffer updates** - GPU uploads are expensive
3. **Batch buffer creation** - Reduce driver overhead
4. **Reuse buffers** - Avoid create/destroy churn
5. **Align data properly** - Use `#[repr(C)]` and padding for uniforms
6. **Profile buffer usage** - Check actual GPU memory with `buffer_size()`

## Common Pitfalls

### 1. Using Destroyed Handles

```rust
let handle = backend.create_buffer(BufferType::Vertex, BufferUsage::Static, &data)?;
backend.destroy_buffer(handle);

// ERROR: Handle is now invalid
backend.bind_buffer(handle)?;  // Returns InvalidHandle error
```

### 2. Out-of-Bounds Updates

```rust
let buffer = backend.create_buffer(BufferType::Vertex, BufferUsage::Dynamic, &[0u8; 16])?;

// ERROR: Trying to write beyond buffer size
backend.update_buffer(buffer, 12, &[1, 2, 3, 4, 5, 6, 7, 8])?;  // 12 + 8 > 16
```

### 3. Updating Static Buffers

```rust
// Works but inefficient
let buffer = backend.create_buffer(BufferType::Vertex, BufferUsage::Static, &data)?;
backend.update_buffer(buffer, 0, &new_data)?;  // Slow! Use Dynamic instead
```

## Next Steps

- **Step 5.1.4**: Texture loading and management
- **Step 5.1.5**: Shader compilation and linking
- **Step 5.1.6**: Draw call submission (arrays, indexed, instanced)

## See Also

- `RenderBackend` trait documentation
- `BufferHandle` type documentation
- Error handling guide in `core::error` module
