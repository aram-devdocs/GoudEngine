use game::{Game, KeyInput, WindowBuilder};
use std::rc::Rc;
use std::cell::RefCell;

struct Player {
    position: (f32, f32),
    speed: f32,
    rotation: f32, // New field for rotation
}

impl Player {
    fn new() -> Self {
        Player {
            position: (0.0, 0.0), // Starting position at the center
            speed: 0.05,          // Adjust speed as needed
            rotation: 0.0,        // Initial rotation
        }
    }

    fn update_position(&mut self, game: &Game) {
        if game.window.input_handler.is_key_pressed(KeyInput::W) {
            self.position.1 += self.speed;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::S) {
            self.position.1 -= self.speed;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::A) {
            self.position.0 -= self.speed;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::D) {
            self.position.0 += self.speed;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::Q) {
            self.rotation -= 0.05; // Rotate counter-clockwise
        }
        if game.window.input_handler.is_key_pressed(KeyInput::E) {
            self.rotation += 0.05; // Rotate clockwise
        }
    }
}

fn main() {
    let mut game = Game::new(WindowBuilder {
        width: 800,
        height: 600,
        title: "Sprite Example".to_string(),
    });

    // Initialize game with custom logic
    game.init(|game| {
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

    // Wrap player in Rc<RefCell> for shared mutability
    let player = Rc::new(RefCell::new(Player::new()));

    let player_clone = Rc::clone(&player);
    game.run(move |game| {
        // Update player position and rotation based on input
        player_clone.borrow_mut().update_position(game);

        // Render logic would use player.position and player.rotation to set sprite location and rotation here
        game.renderer.as_mut().unwrap().update_sprite_transform(
            0,
            cgmath::Vecutor2::new(player_clone.borrow().position.0, player_clone.borrow().position.1),
            player_clone.borrow().rotation,
        );

        // Exit game if Escape is pressed
        if game.window.input_handler.is_key_pressed(KeyInput::Escape) {
            game.window.close_window();
        }
    });
}