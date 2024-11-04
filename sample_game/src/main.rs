use game::{Game, TriangleProps};
use platform::graphics::{gl_wrapper::VertexAttributeProps, window::WindowBuilder};

fn main() {
    let mut game = Game::new(WindowBuilder {
        width: 1280,
        height: 720,
        title: "Sample Game".to_string(),
    });

    // Initialize game
    game.init();

    // Create and add a triangle to the game
    game.create_triangle(
        TriangleProps {
            vertices: [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0],
        },
        VertexAttributeProps {
            index: 0,
            size: 3,
            r#type: gl::FLOAT,
            normalized: gl::FALSE,
            stride: 0,
            pointer: std::ptr::null(),
        },
    );

    // Run the game loop
    game.run();
}
