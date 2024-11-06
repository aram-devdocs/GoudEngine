# GoudEngine

GoudEngine is a game engine written in Rust, designed to be used with C# applications. It provides a set of tools and libraries for creating 2D and 3D\* games.

## Features

- 2D and 3D\* rendering
- Sprite management
- Input handling
- Window management
- Graphics rendering with OpenGL
- Integration with C# using csbindgen

## Installation

To install the necessary dependencies and tools, run the following script:

```sh
./install.sh
```

This script will install Rust and the required Rust components if they are not already installed.

Building the Project
To build the project in release mode, run the following script:

```sh
./build.sh
```

This script will build the project and copy the generated dynamic library to the flappy_goud directory.

## Sample

To run the sample game, `flappy_goud`, run the following script:

```sh
cd flappy_goud
dotnet build
dotnet run
```

This script will build and run the sample game. The entry point to that game is located in `flappy_goud/Program.cs`.

## Usage

Rust Library
The Rust library provides the core functionality of the game engine. The main entry point is the goud_engine crate.

Example:

```rust
mod game;

use game::cgmath::Vector2;
use game::{GameSdk, Rectangle, Sprite, Texture, WindowBuilder};
use glfw::Key;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[no_mangle]
pub extern "C" fn game_create(width: u32, height: u32, title: *const c_char) -> *mut GameSdk {
    println!("Creating game instance");
    let title_str = unsafe { CStr::from_ptr(title).to_str().unwrap() };
    let title_cstring = CString::new(title_str).unwrap();
    let builder = WindowBuilder {
        width,
        height,
        title: title_cstring.as_ptr(),
    };
    let game = GameSdk::new(builder);
    Box::into_raw(Box::new(game))
}

// ... other functions ...
```

Functions and structs in this file along with a few others in the `goud_engine` model are exposed to C# using `csbindgen`, and generated as libgoud_engine*.dll*.dylib*.so*. The generated library is then used in the C# application with NativeMethods.g.cs.

C# Integration
The C# integration allows you to use the Rust game engine in your C# applications. The main entry point is the GoudGame class.

Example

```csharp
using System;

class Program
{
    static void Main(string[] args)
    {
        GoudGame game = new GoudGame(800, 600, "Flappy Bird");

        GameManager gameManager; // Created for sample project

        game.Initialize(() =>
        {
            Console.WriteLine("Game Initialized!");

            GoudGame.SpriteData backgroundData = new GoudGame.SpriteData { X = 0, Y = 0, ScaleX = 1, ScaleY = 1, Rotation = 0 };
            game.AddSprite("assets/sprites/background-day.png", backgroundData);
        });

        gameManager = new GameManager(game); // Created for sample project

        game.Start(() =>
        {
            Console.WriteLine("Game Started!");
        });

        game.Update(() =>
        {
            gameManager.Update(); // Created for sample project
        });

        game.Terminate();
    }
}
```

The GoudGame class is a wrapper around the Rust game engine, providing a C# interface to the engine. There are examples in `flappy_goud` on expanding modules using the base GoudGame class, but the beauty of this project is that you can create your own game engine modules in Rust and use them in C# applications.

Documentation
official documentation of `csbindgen` [here](https://github.com/mozilla/cbindgen).

## License

This project is licensed under the MIT License - see the LICENSE file for details.
