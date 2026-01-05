# Goud Engine Unit Testing Implementation Plan

## Overview

This document contains a comprehensive plan for achieving 100% unit test coverage for the Goud Engine and SDK. The tasks are organized into phases that can be executed in parallel, followed by QA phases.

## Current Testing Status


## Testing Conventions Established

- Test modules use `#[cfg(test)]` and `mod tests`
- Test functions follow pattern: `test_<struct>_<method>()`
- Mock implementations for OpenGL-dependent code
- Conditional testing based on OpenGL context availability

---

## Phase 2: OpenGL-Dependent Components (Can be parallelized)

### Ticket 2.1: Unit Tests for Buffer and VAO Components

**Files**: buffer.rs, vao.rs
**Priority**: High
**Estimated Effort**: 8 hours

**Context**: These components heavily depend on OpenGL. Testing strategy should use dependency injection or trait-based mocking.

**Requirements**:

1. Add unit tests to `/goud_engine/src/libs/graphics/components/buffer.rs`:

   - Create a trait for OpenGL functions to enable mocking
   - Test successful buffer creation scenarios
   - Test error handling when OpenGL operations fail
   - Test bind/unbind state management
   - Test data upload with different types (f32, u32)
   - Test buffer deletion and cleanup

2. Add unit tests to `/goud_engine/src/libs/graphics/components/vao.rs`:
   - Mock OpenGL VAO functions
   - Test VAO creation and binding
   - Test attribute enabling/disabling
   - Test cleanup on drop

**Testing Pattern**: Create mock OpenGL traits similar to MockShaderProgram pattern. Consider using the existing test_helper module.

---

### Ticket 2.2: Unit Tests for Skybox Component

**Files**: skybox.rs
**Priority**: Medium
**Estimated Effort**: 6 hours

**Context**: Skybox manages cube map textures and rendering state for 3D environments.

**Requirements**:

1. Add unit tests to `/goud_engine/src/libs/graphics/components/skybox.rs`:
   - Test skybox initialization with texture paths
   - Test rendering state management
   - Test uniform updates for view/projection matrices
   - Mock texture loading and OpenGL texture operations
   - Test error handling for missing textures

**Testing Pattern**: Use mocks for texture loading and shader operations.

---

### Ticket 2.3: Unit Tests for Texture Manager

**Files**: texture_manager.rs
**Priority**: High
**Estimated Effort**: 6 hours

**Context**: TextureManager handles texture caching and lifecycle management.

**Requirements**:

1. Add unit tests to `/goud_engine/src/libs/graphics/components/textures/texture_manager.rs`:
   - Test texture loading and caching
   - Test duplicate texture handling
   - Test texture cleanup and memory management
   - Test error handling for invalid paths
   - Mock file system operations and OpenGL texture creation

**Testing Pattern**: Mock file system and OpenGL operations. Test caching logic thoroughly.

---

## Phase 3: Renderer Components (Sequential - Start after Phase 2)

### Ticket 3.1: Unit Tests for Base Renderer

**Files**: renderer.rs
**Priority**: High
**Estimated Effort**: 5 hours

**Context**: The base renderer provides abstraction over renderer2d and renderer3d without direct OpenGL calls.

**Requirements**:

1. Add unit tests to `/goud_engine/src/libs/graphics/renderer.rs`:
   - Test renderer initialization
   - Test unsafe pointer operations with proper mocking
   - Test delegation to renderer2d/renderer3d
   - Test error handling and state management
   - Test memory safety of pointer conversions

**Testing Pattern**: This is one of the few renderer files that can be unit tested without OpenGL context.

---

### Ticket 3.2: Integration Test Framework for Renderer2D/3D

**Files**: renderer2d.rs, renderer3d.rs
**Priority**: Medium
**Estimated Effort**: 10 hours

**Context**: These renderers are heavily OpenGL-dependent and require integration testing approach.

**Requirements**:

1. Create integration test framework for renderers:
   - Set up test harness with OpenGL context
   - Extract testable pure functions where possible
   - Test camera state management separately
   - Test sprite batching logic (renderer2d)
   - Test light and skybox integration (renderer3d)
   - Create minimal rendering scenarios

**Testing Pattern**: Focus on integration tests rather than unit tests. Extract pure logic where possible.

---

## Phase 4: Game Logic and SDK (Can be parallelized)

### Ticket 4.1: Unit Tests for Game Module

**Files**: game.rs
**Priority**: High
**Estimated Effort**: 8 hours

**Context**: The game module manages sprites, tilesets, and integrates multiple systems.

**Requirements**:

1. Add unit tests to `/goud_engine/src/game.rs`:
   - Extract tileset management logic into testable functions
   - Test sprite ID generation and tracking
   - Test sprite cleanup and memory management
   - Mock ECS, TextureManager, and TiledManager dependencies
   - Test error handling for resource loading
   - Test game state initialization and cleanup

**Testing Pattern**: Heavy use of mocking for external dependencies. Focus on state management logic.

---

### Ticket 4.2: C# SDK Unit Tests - Core Classes

**Files**: GoudGame.cs, Animation.cs, AnimationController.cs
**Priority**: High
**Estimated Effort**: 8 hours

**Context**: The C# SDK has zero test coverage. Need to establish testing framework and patterns.

**Requirements**:

1. Set up C# testing framework (MSTest or NUnit)
2. Add unit tests for `GoudGame.cs`:

   - Test initialization and configuration
   - Test game loop methods
   - Mock P/Invoke calls to native library
   - Test error handling

3. Add unit tests for `Animation.cs`:

   - Test animation frame management
   - Test timing and interpolation
   - Test state transitions

4. Add unit tests for `AnimationController.cs`:
   - Test animation registration and lookup
   - Test animation switching logic
   - Test edge cases (missing animations, invalid states)

**Testing Pattern**: Use mocking framework (Moq) for P/Invoke calls. Establish C# testing conventions.

---

### Ticket 4.3: Unit Tests for SDK Entry Point

**Files**: sdk.rs, lib.rs
**Priority**: Medium
**Estimated Effort**: 4 hours

**Context**: These files provide FFI boundaries and library initialization.

**Requirements**:

1. Add unit tests to `/goud_engine/src/sdk.rs`:

   - Test FFI function signatures
   - Test parameter validation
   - Test error code returns

2. Add unit tests to `/goud_engine/src/lib.rs`:
   - Test library initialization
   - Test module exports
   - Test panic handling at FFI boundary

**Testing Pattern**: Focus on boundary testing and error handling.

---

## Phase 5: Module Organization Files (Low Priority)

### Ticket 5.1: Unit Tests for Module Files

**Files**: All mod.rs files (8 files)
**Priority**: Low
**Estimated Effort**: 2 hours

**Context**: Module files typically only contain re-exports but may have initialization logic.

**Requirements**:

1. Add tests where module files contain logic beyond re-exports
2. Test any lazy_static or initialization code
3. Verify module visibility and exports

**Testing Pattern**: Only add tests if modules contain actual logic.

---

## Phase 6: Quality Assurance (Sequential - After all phases)

### Ticket 6.1: Test Coverage Analysis and Gap Filling

**Priority**: High
**Estimated Effort**: 8 hours

**Requirements**:

1. Run coverage tools (cargo-tarpaulin for Rust, dotCover for C#)
2. Identify uncovered code paths
3. Add tests for edge cases and error conditions
4. Ensure all public APIs have tests
5. Document any intentionally untested code

---

### Ticket 6.2: Test Quality Review

**Priority**: High
**Estimated Effort**: 6 hours

**Requirements**:

1. Review all tests for:
   - Proper assertions
   - Good test names
   - No test interdependencies
   - Adequate edge case coverage
2. Ensure mock implementations are accurate
3. Verify tests actually test behavior, not implementation
4. Add property-based tests where appropriate

---

### Ticket 6.3: CI/CD Integration

**Priority**: High
**Estimated Effort**: 4 hours

**Requirements**:

1. Set up automated test running in CI
2. Configure coverage reporting
3. Set minimum coverage thresholds (aim for 80%+)
4. Ensure tests run without GPU in CI environment
5. Add test status badges to README

---

## Summary

- **Total Files Needing Tests**: 29 (24 Rust + 5 C#)
- **Estimated Total Effort**: ~100 hours
- **Phases**: 6 (4 parallel implementation phases + 2 sequential QA phases)
- **Priority Order**: Core components → OpenGL components → Renderers → Game/SDK → Modules → QA

## Success Criteria

1. Minimum 80% code coverage across the project
2. All public APIs have comprehensive tests
3. Tests run successfully in CI without GPU
4. Clear documentation of testing patterns
5. Established mocking patterns for OpenGL dependencies
