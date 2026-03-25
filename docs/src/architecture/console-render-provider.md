# Console Render Backend Contract

This guide is for NDA partners who need to plug a proprietary renderer into the
public engine. It documents the live `RenderProvider` trait in
`goud_engine/src/core/providers/render.rs` and keeps the public side of the
integration clear. Platform SDK calls, swap-chain setup details, and command
buffer code stay in your private repo.

## What the Engine Expects

The engine owns gameplay, scene traversal, asset descriptors, and the frame
loop. Your backend owns GPU device setup, surface management, command
submission, and presentation.

The public contract is:

- `RenderProvider` defines rendering operations.
- `Provider` identifies the backend and exposes capabilities.
- `ProviderLifecycle` covers startup, per-frame maintenance, and shutdown.

The engine talks only through these traits. It does not call console graphics
APIs directly.

## Integration Points

| Concern | Public hook | Backend responsibility |
|---|---|---|
| Window or surface bootstrap | provider constructor plus `init()` | Bind the renderer to the partner window, surface, or swap-chain object |
| Back-buffer acquire and present | `begin_frame()` / `end_frame()` | Acquire the current target, record commands, submit, and present |
| Resize or mode change | `resize(width, height)` | Rebuild the swap chain, cached render targets, and dependent state |
| GPU work submission | draw and resource methods | Translate engine descriptors into private API calls |
| Diagnostics | `render_diagnostics()` | Report counts and timings the engine can surface in tools |

If the console runtime requires extra state that is not part of the public API,
store it inside your provider struct and wire it through your private crate.

## Base Trait Rules

### `Provider`

Implement `name()`, `version()`, and `capabilities()` with stable values.

- `name()` should identify the backend, not the platform holder. Examples:
  `"console-vulkan"`, `"partner-gpu"`.
- `version()` should track the backend package or integration version.
- `capabilities()` should return the same data as `render_capabilities()` in a
  type-erased form.

### `ProviderLifecycle`

The lifecycle is `create -> init -> update -> shutdown -> drop`.

- `init()` should allocate device objects that must exist before the first
  frame.
- `update(delta)` is available for per-frame maintenance. Use it for fence
  cleanup, deferred destruction, or stats rollover if your backend needs it.
- `shutdown()` must release GPU and OS resources without panicking.

Keep `shutdown()` idempotent if your private integration layer may call it more
than once during teardown.

## `RenderProvider` Method Contracts

### Capabilities

`render_capabilities(&self) -> &RenderCapabilities`

- Fill in the real limits for texture size, batching, render targets, MSAA, and
  instancing.
- Set `backend_name` to a string that tells the game or tooling which backend
  is live.
- Treat these values as stable for the life of the provider.

### Frame lifecycle

`begin_frame()` starts work for one frame and returns a `FrameContext`.

- Acquire the back buffer or current render target here.
- Reset transient allocators and frame-local command state here.
- Return an error if the frame cannot start cleanly.

`end_frame(frame)` completes the same frame.

- Consume the `FrameContext` you returned from `begin_frame()`.
- Flush pending work, submit commands, and present.
- Release frame-local resources that are tied to the acquired image.

`resize(width, height)` handles any display-size change.

- Rebuild swap-chain or render-target state that depends on framebuffer size.
- Preserve persistent resources when possible.
- Return an error if the new size cannot be supported.

### Resource management

The engine creates opaque handles and expects the backend to keep the real GPU
objects behind them.

- `create_texture`, `create_buffer`, `create_shader`, `create_pipeline`, and
  `create_render_target` should allocate backend resources and return stable
  handles.
- The paired `destroy_*` methods must release the matching resource.
- Keep ownership local to the provider. Do not leak raw backend pointers across
  the public boundary.
- If your backend defers destruction until the GPU is idle, queue the work
  internally and retire it from `update()` or `end_frame()`.

### Drawing

The engine sends already-resolved draw descriptors. Your job is to map them to
private API calls.

- `draw(&DrawCommand)` handles a single 2D draw.
- `draw_batch(&[DrawCommand])` should preserve the batch order supplied by the
  engine.
- `draw_mesh(&MeshDrawCommand)` covers 3D geometry.
- `draw_text(&TextDrawCommand)` covers glyph submission after layout.
- `draw_particles(&ParticleDrawCommand)` covers particle draws.

Treat these methods as command recording or direct submission hooks. Do not
present from them; presentation belongs in `end_frame()`.

### Render state

- `set_viewport(x, y, width, height)` updates viewport state.
- `set_camera(camera)` updates the active view and projection data.
- `set_render_target(handle)` switches between the default framebuffer and an
  off-screen target.
- `render_target_texture(handle)` should return the texture handle bound to an
  off-screen target when your backend supports that mapping.
- `clear(color)` clears the current target.

You can cache this state internally if your backend batches state changes.

### Diagnostics

`render_diagnostics()` should return a cheap snapshot.

Useful fields include:

- draw-call counts
- buffer uploads
- frame time buckets
- resource counts

These numbers do not need profiler precision. They do need to match what the
backend actually submitted.

## Skeleton Implementation

```rust
use goud_engine::core::error::GoudResult;
use goud_engine::core::providers::diagnostics::RenderDiagnosticsV1;
use goud_engine::core::providers::render::RenderProvider;
use goud_engine::core::providers::types::{
    BufferDesc, BufferHandle, CameraData, DrawCommand, FrameContext,
    MeshDrawCommand, ParticleDrawCommand, PipelineDesc, PipelineHandle,
    RenderCapabilities, RenderTargetDesc, RenderTargetHandle, ShaderDesc,
    ShaderHandle, TextDrawCommand, TextureDesc, TextureHandle,
};
use goud_engine::core::providers::{Provider, ProviderLifecycle};

pub struct ConsoleRenderProvider {
    caps: RenderCapabilities,
    backend: PrivateConsoleBackend,
}

impl ConsoleRenderProvider {
    pub fn new(backend: PrivateConsoleBackend, caps: RenderCapabilities) -> Self {
        Self { backend, caps }
    }
}

impl Provider for ConsoleRenderProvider {
    fn name(&self) -> &str { "partner-console" }
    fn version(&self) -> &str { "1.0.0" }
    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.caps.clone())
    }
}

impl ProviderLifecycle for ConsoleRenderProvider {
    fn init(&mut self) -> GoudResult<()> {
        self.backend.init_device()
    }

    fn update(&mut self, delta: f32) -> GoudResult<()> {
        self.backend.retire_completed_work(delta)
    }

    fn shutdown(&mut self) {
        self.backend.shutdown();
    }
}

impl RenderProvider for ConsoleRenderProvider {
    fn render_capabilities(&self) -> &RenderCapabilities { &self.caps }

    fn begin_frame(&mut self) -> GoudResult<FrameContext> {
        self.backend.begin_frame()
    }

    fn end_frame(&mut self, frame: FrameContext) -> GoudResult<()> {
        self.backend.end_frame(frame)
    }

    fn resize(&mut self, width: u32, height: u32) -> GoudResult<()> {
        self.backend.resize(width, height)
    }

    fn create_texture(&mut self, desc: &TextureDesc) -> GoudResult<TextureHandle> {
        self.backend.create_texture(desc)
    }

    fn destroy_texture(&mut self, handle: TextureHandle) {
        self.backend.destroy_texture(handle);
    }

    fn create_buffer(&mut self, desc: &BufferDesc) -> GoudResult<BufferHandle> {
        self.backend.create_buffer(desc)
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) {
        self.backend.destroy_buffer(handle);
    }

    fn create_shader(&mut self, desc: &ShaderDesc) -> GoudResult<ShaderHandle> {
        self.backend.create_shader(desc)
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) {
        self.backend.destroy_shader(handle);
    }

    fn create_pipeline(&mut self, desc: &PipelineDesc) -> GoudResult<PipelineHandle> {
        self.backend.create_pipeline(desc)
    }

    fn destroy_pipeline(&mut self, handle: PipelineHandle) {
        self.backend.destroy_pipeline(handle);
    }

    fn create_render_target(
        &mut self,
        desc: &RenderTargetDesc,
    ) -> GoudResult<RenderTargetHandle> {
        self.backend.create_render_target(desc)
    }

    fn destroy_render_target(&mut self, handle: RenderTargetHandle) {
        self.backend.destroy_render_target(handle);
    }

    fn draw(&mut self, cmd: &DrawCommand) -> GoudResult<()> {
        self.backend.draw(cmd)
    }

    fn draw_batch(&mut self, cmds: &[DrawCommand]) -> GoudResult<()> {
        self.backend.draw_batch(cmds)
    }

    fn draw_mesh(&mut self, cmd: &MeshDrawCommand) -> GoudResult<()> {
        self.backend.draw_mesh(cmd)
    }

    fn draw_text(&mut self, cmd: &TextDrawCommand) -> GoudResult<()> {
        self.backend.draw_text(cmd)
    }

    fn draw_particles(&mut self, cmd: &ParticleDrawCommand) -> GoudResult<()> {
        self.backend.draw_particles(cmd)
    }

    fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.backend.set_viewport(x, y, width, height);
    }

    fn set_camera(&mut self, camera: &CameraData) {
        self.backend.set_camera(camera);
    }

    fn set_render_target(&mut self, handle: Option<RenderTargetHandle>) {
        self.backend.set_render_target(handle);
    }

    fn clear(&mut self, color: [f32; 4]) {
        self.backend.clear(color);
    }

    fn render_diagnostics(&self) -> RenderDiagnosticsV1 {
        self.backend.render_diagnostics()
    }
}
```

The `PrivateConsoleBackend` type is yours. Keep it outside the public repo if
it contains NDA APIs, private handles, or platform-specific build glue.

## Common Failure Modes

- Presenting from `draw_*` instead of `end_frame()`
- Rebuilding swap-chain state in `begin_frame()` on every frame instead of only
  on resize or recoverable acquire failure
- Returning guessed capabilities instead of measured limits
- Letting resource handles escape into partner-owned game code
- Reporting diagnostics from queued work rather than submitted work

## Related Guides

- [Provider System](providers.md)
- [Console Porting Guide](../guides/console-porting.md)
