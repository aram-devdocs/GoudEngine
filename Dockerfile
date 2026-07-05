# Multi-stage build for GoudEngine.
#
#   builder  - compiles the workspace with a locked Cargo.lock (reproducible).
#   test     - adds a software Vulkan stack (Mesa lavapipe) so GPU-dependent
#              tests and headless render checks run without a hardware GPU.
#   runtime  - minimal image carrying only the built library and CA certs.
#
# The pinned base matches rust-toolchain.toml; the workspace toolchain file
# refines the exact patch version at build time.

# ---- builder ---------------------------------------------------------------
FROM rust:1.95-slim-bookworm AS builder
WORKDIR /src

# System libraries the engine links against (mirrors the CI Ubuntu deps).
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libgl1-mesa-dev libglu1-mesa-dev \
        libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libxxf86vm-dev \
        libasound2-dev libudev-dev \
        python3 \
    && rm -rf /var/lib/apt/lists/*

COPY . .
# --locked enforces the committed Cargo.lock so the image is reproducible.
RUN cargo build --release --locked -p goud_engine

# ---- test ------------------------------------------------------------------
# Adds a software rasterizer so `cargo test` and headless render checks run
# with no hardware GPU present. Mesa lavapipe provides a Vulkan ICD that wgpu
# selects via force_fallback_adapter (GOUD_WGPU_FORCE_FALLBACK=1).
FROM builder AS test
RUN apt-get update && apt-get install -y --no-install-recommends \
        mesa-vulkan-drivers vulkan-tools \
        xvfb \
    && rm -rf /var/lib/apt/lists/*

ENV LIBGL_ALWAYS_SOFTWARE=1 \
    GALLIUM_DRIVER=llvmpipe \
    VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json \
    GOUD_WGPU_FORCE_FALLBACK=1 \
    GOUD_SKIP_NATIVE_SMOKE=1 \
    CI=1

# Default: run the headless-safe test suite under a virtual framebuffer.
CMD ["bash", "-lc", "xvfb-run -a cargo test --release -p goud_engine"]

# ---- runtime ---------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates libgl1 libasound2 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/libgoud_engine.* /usr/local/lib/
