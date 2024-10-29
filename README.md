# GoudEngine

GoudEngine is a modular, cross-platform 2D game engine written in C++ with future C# interoperability.

## Building the Engine

### Prerequisites

- C++17 compiler
- CMake 3.15 or higher
- Git

### Build Steps

```bash
git clone https://your-repo-url.git
cd GoudEngine
mkdir build
cd build
cmake ..
cmake --build .
```

### Running the Sample

After building, you can run the sample application:

```bash
./samples/BasicSample/BasicSample
```

Project Structure

	•	engine/: Core engine code.
	•	modules/: Modular components like graphics, audio, etc.
	•	third_party/: External dependencies.
	•	samples/: Sample applications.
	•	tests/: Test suites.
	•	docs/: Documentation.
	•	tools/: Utility tools and scripts.
