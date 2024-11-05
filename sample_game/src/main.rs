// main.rs

use game::{Game, KeyInput, Rectangle, Sprite, Texture, WindowBuilder};
use std::cell::RefCell;
use std::rc::Rc;

use game::cgmath::Vector2;


struct Player {
    position: Vector2<f32>,
    speed: f32,
    rotation: f32,
    scale: Vector2<f32>,
}

impl Player {
    fn new() -> Self {
        Player {
            position: Vector2::new(0.0, 0.0),
            speed: 0.05,
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
        }
    }

    fn update(&mut self, game: &Game) {
        if game.window.input_handler.is_key_pressed(KeyInput::W) {
            self.position.y += self.speed;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::S) {
            self.position.y -= self.speed;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::A) {
            self.position.x -= self.speed;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::D) {
            self.position.x += self.speed;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::Q) {
            self.rotation -= 0.05;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::E) {
            self.rotation += 0.05;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::Z) {
            self.scale.x -= 0.01;
            self.scale.y -= 0.01;
        }
        if game.window.input_handler.is_key_pressed(KeyInput::X) {
            self.scale.x += 0.01;
            self.scale.y += 0.01;
        }
    }
}

fn main() {
    let mut game = Game::new(WindowBuilder {
        width: 800,
        height: 600,
        title: "Sprite Example".to_string(),
    });

    // Wrap player in Rc<RefCell> for shared mutability
    let player = Rc::new(RefCell::new(Player::new()));
    player.borrow_mut().position = Vector2::new(0.0, 0.0);
    player.borrow_mut().scale = Vector2::new(0.1, 0.1);
    let player_clone = Rc::clone(&player);

    // Initialize game with custom logic
    game.init(|game| {
        // Load texture
        let texture = Texture::new("sample_game/assets/bluebird-midflap.png")
            .expect("Failed to load texture");

        // Define source rectangle (portion of the spritesheet)
        let source_rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1.0,  // Adjust width as needed
            height: 1.0, // Adjust height as needed
        };

        // Create sprite
        let sprite = Sprite::new(
            texture,
            player_clone.borrow().position,
            player_clone.borrow().scale,
            player_clone.borrow().rotation,
            Some(source_rect),
        );

        // Add sprite to renderer
        game.renderer_2d.as_mut().unwrap().add_sprite(sprite);
    });

    game.run(move |game| {
        // Update player
        player_clone.borrow_mut().update(game);

        // Update sprite in renderer
        let sprite = Sprite::new(
            game.renderer_2d.as_ref().unwrap().sprites[0]
                .texture
                .clone(),
            player_clone.borrow().position,
            player_clone.borrow().scale,
            player_clone.borrow().rotation,
            Some(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 1.0,  // Adjust width as needed
                height: 1.0, // Adjust height as needed
            }),
        );

        game.renderer_2d
            .as_mut()
            .unwrap()
            .update_sprite(0, sprite)
            .expect("Failed to update sprite");

        // Exit game if Escape is pressed
        if game.window.input_handler.is_key_pressed(KeyInput::Escape) {
            game.window.close_window();
        }

        // if R is pressed, reset player position
        if game.window.input_handler.is_key_pressed(KeyInput::R) {
            player_clone.borrow_mut().position = Vector2::new(0.0, 0.0);
            player_clone.borrow_mut().scale = Vector2::new(0.1, 0.1);
            player_clone.borrow_mut().rotation = 0.0;
        }
    });
}
