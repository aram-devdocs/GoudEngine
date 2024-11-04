// main.rs

use game::{Game, WindowBuilder};

fn main() {
    let mut game = Game::new(WindowBuilder {
        width: 800,
        height: 600,
        title: "Sprite Example".to_string(),
    });

    // Initialize game with custom logic
    game.init(|game| {
        // Define the vertices and indices for a textured quad
        let vertices: Vec<f32> = vec![
            // positions       // texture coords
            0.5, 0.5, 0.0, 1.0, 1.0, // top right
            0.5, -0.5, 0.0, 1.0, 0.0, // bottom right
            -0.5, -0.5, 0.0, 0.0, 0.0, // bottom left
            -0.5, 0.5, 0.0, 0.0, 1.0, // top left
        ];

        let indices: Vec<u32> = vec![
            0, 1, 3, // first triangle
            1, 2, 3, // second triangle
        ];

        // Add sprite to renderer
        game.renderer.as_mut().unwrap().add_sprite(
            &vertices,
            &indices,
            "sample_game/assets/bluebird-midflap.png",
        );
    });

    game.run(|game| {
        // Handle input, update game state, etc.

        // For example, move sprites, handle keyboard input, etc.
        if game
            .window
            .input_handler
            .is_key_pressed(game::KeyInput::Escape)
        {
            game.window.close_window();
        }
    });
}
