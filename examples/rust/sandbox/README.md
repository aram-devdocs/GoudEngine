# Rust Sandbox

Interactive sandbox parity example for the Rust SDK.

Run from the repository root:

```bash
cargo run -p sandbox
```

To force the legacy native path instead of the default `winit + wgpu` stack:

```bash
GOUD_NATIVE_BACKEND=legacy cargo run -p sandbox
```
