# GoudEngine TODO

## Project 1: Define Engine Goals and Architecture

- [ ] **Establish Core Objectives**
  - [ ] Purpose: Create a modular, non-opinionated, code-first 2D game engine with built-in networking capabilities.
  - [ ] Flexibility: Ensure the engine can run on clients, servers, or any combination, allowing developers to choose their architecture.
  - [ ] Cross-Platform Support: Aim for “build once, deploy anywhere” across Windows, macOS, Linux, iOS, Android, and major consoles.
- [ ] **Design Modular Architecture**
  - [ ] Monorepo Structure: Organize code in a monorepository with clear separation of concerns into different libraries/modules.
  - [ ] Pyramid Dependency Structure: Start from high-level systems and work down to atomic components.
  - [ ] Open-Source Compliance: Use only open-source libraries to ensure broad availability and compliance.

## Project 2: Set Up Development Environment

- [ ] **Toolchain Selection**
  - [ ] Programming Languages: Use C++ for the engine core and C# for game development.
  - [ ] Graphics API: Choose between OpenGL, SDL2, or Vulkan as the base graphics API, favoring open-source options.
  - [ ] Build Systems: Configure cross-platform build systems (e.g., CMake) to handle multiple platforms.
- [ ] **Monorepo Configuration**
  - [ ] Version Control: Initialize a Git repository with branching strategies suitable for collaborative development.
  - [ ] Continuous Integration: Set up CI pipelines for automated testing and building on all target platforms.

## Project 3: Cross-Platform Foundation

- [ ] **Platform Abstraction Layer**
  - [ ] Unified Interfaces: Develop abstraction layers for graphics, input, audio, and networking to handle platform-specific details.
  - [ ] Scalability: Ensure the abstraction layer supports scaling from single-player to massive multiplayer online games.
- [ ] **Third-Party Library Integration**
  - [ ] Graphics: Integrate OpenGL, SDL2, or Vulkan for rendering.
  - [ ] Input and Audio: Use open-source libraries like SDL2 for input and audio handling.
  - [ ] Networking: Incorporate cross-platform networking libraries (e.g., ENet or Asio).

## Project 4: Core Systems Development

- [ ] **Graphics System**
  - [ ] Rendering Engine: Implement a 2D rendering engine with support for sprites, tilemaps, and shaders.
  - [ ] Shader Support: Allow for custom shaders to enable advanced visual effects.
  - [ ] Camera Control: Provide systems for camera movement, zoom, and viewport management.
- [ ] **Input System**
  - [ ] Device Support: Handle input from keyboards, mice, touchscreens, and game controllers.
  - [ ] Event Handling: Create an event-driven system to process input consistently across platforms.
- [ ] **Audio System**
  - [ ] Playback Capabilities: Implement audio playback for music and sound effects.
  - [ ] Format Support: Ensure compatibility with common audio formats like WAV, MP3, and OGG.

## Project 5: Networking System

- [ ] **Networking Architecture**
  - [ ] Flexible Topologies: Design the system to support client-server, peer-to-peer, and hybrid models.
  - [ ] Synchronization: Implement mechanisms for synchronizing game states between clients and servers.
  - [ ] Data Serialization: Develop efficient serialization methods for network communication.
- [ ] **Modularity**
  - [ ] Network Layer Abstraction: Allow developers to use or replace the networking module as needed.
  - [ ] Server and Client Interoperability: Ensure that components can run interchangeably on servers or clients.

## Project 6: Physics and Collision System

- [ ] **Physics Engine**
  - [ ] 2D Physics: Implement or integrate an open-source physics library for realistic movement and interactions.
  - [ ] Collision Detection: Develop systems for detecting and resolving collisions between entities.
- [ ] **Modular Design**
  - [ ] Flexible Deployment: Allow physics calculations to occur on the client, server, or both.
  - [ ] Customization: Enable developers to extend or replace the physics module.

## Project 7: Entity Component System (ECS)

- [ ] **ECS Framework**
  - [ ] Non-Opinionated Design: Create an ECS that doesn’t enforce a specific coding style.
  - [ ] Performance: Optimize for handling large numbers of entities efficiently.
- [ ] **Controllers and Services**
  - [ ] Entity Controllers: Provide high-level controllers for common entity behaviors.
  - [ ] Service Layer: Expose mid-level APIs for animation, input handling, and other services.

## Project 8: Resource Management

- [ ] **Asset Loading System**
  - [ ] Asynchronous Loading: Implement non-blocking asset loading to improve performance.
  - [ ] Resource Caching: Develop caching mechanisms to reuse assets efficiently.
- [ ] **Memory Management**
  - [ ] Custom Allocators: Optimize memory usage with custom allocation strategies.
  - [ ] Profiling Tools: Include tools to monitor and debug memory usage.

## Project 9: Utilities and Quality of Life Features

- [ ] **Math Library**
  - [ ] Core Mathematics: Provide vectors, matrices, quaternions, and transformation utilities.
  - [ ] Extended Functions: Include collision math, interpolation, and random number generation.
- [ ] **Animation System**
  - [ ] Animation Controller: Build on the animation service to manage complex animations.
  - [ ] Skeletal and Sprite Animations: Support multiple animation types.
- [ ] **Entity Systems**
  - [ ] Entity Controller: Develop controllers for entity lifecycle management.
  - [ ] Component Management: Simplify adding, removing, and updating components.

## Project 10: Interfacing with C#

- [ ] **C API Exposure**
  - [ ] C-Compatible Interface: Expose engine functionalities through a C API for language interoperability.
  - [ ] P/Invoke Support: Design the API to be easily callable from C# using P/Invoke.
- [ ] **C# SDK Development**
  - [ ] Managed Wrapper: Create a C# library that wraps the C API, providing a natural C# interface.
  - [ ] Object-Oriented Design: Ensure the SDK aligns with C# conventions and best practices.
- [ ] **Cross-Language Data Handling**
  - [ ] Marshaling: Implement data marshaling between C++ and C#.
  - [ ] Memory Safety: Ensure safe memory management across language boundaries.

## Project 11: Cross-Platform Support Extension

- [ ] **Mobile Platforms**
  - [ ] iOS and Android Support: Adapt the engine for mobile devices, handling touch input and performance optimizations.
  - [ ] Platform-Specific Features: Utilize mobile hardware capabilities like accelerometers and gyroscopes.
- [ ] **Console Support**
  - [ ] Research and Planning: Investigate requirements for major consoles (Xbox, PlayStation, Nintendo Switch).
  - [ ] SDK Integration: Prepare for integration with console SDKs, considering licensing and certification.

## Project 12: Testing and Optimization

- [ ] **Automated Testing**
  - [ ] Unit Tests: Write tests for individual components and modules.
  - [ ] Integration Tests: Ensure that different systems work together seamlessly.
- [ ] **Performance Profiling**
  - [ ] Benchmarking: Measure performance across different platforms and configurations.
  - [ ] Optimization: Identify bottlenecks and optimize critical code paths.

## Project 13: Documentation and Examples

- [ ] **Comprehensive Documentation**
  - [ ] API References: Document all functions, classes, and modules with usage examples.
  - [ ] Developer Guides: Provide tutorials and guides for common tasks and best practices.
- [ ] **Sample Projects**
  - [ ] Demonstration Games: Create sample games showcasing engine features and configurations.
  - [ ] Use Case Scenarios: Include examples for single-player, multiplayer, client-server models, and more.

## Project 14: Packaging and Distribution

- [ ] **Build Automation**
  - [ ] Cross-Platform Builds: Develop scripts for building the engine and SDK on all supported platforms.
  - [ ] Continuous Delivery: Set up pipelines for automatic packaging and distribution.
- [ ] **Versioning and Releases**
  - [ ] Semantic Versioning: Adopt a versioning scheme for clear release management.
  - [ ] Release Notes: Document changes, new features, and fixes in each release.
- [ ] **Licensing**
  - [ ] Open-Source License: Choose a license that permits broad use and contribution (e.g., MIT, Apache 2.0).
  - [ ] Compliance: Ensure all third-party libraries are compatible with your chosen license.

## Project 15: Future Enhancements

- [ ] **Editor Development**
  - [ ] Long-Term Goal: Plan for an optional game editor to complement the code-first approach.
  - [ ] Hybrid Workflows: Consider supporting both code-based and visual development styles.
- [ ] **Community and Contributions**
  - [ ] Contribution Guidelines: Establish guidelines for external developers to contribute.
  - [ ] Community Support: Set up forums or chat platforms for user interaction and support.
- [ ] **Plugin and Scripting Support**
  - [ ] Extensibility: Allow for plugins or scripts to extend engine functionality.
  - [ ] Scripting Languages: Consider integrating scripting languages like Lua or Python for rapid prototyping.
