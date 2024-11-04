use game::{Game, TriangleProps, VertexAttributeProps, WindowBuilder};

const VERTEX_ATTRIBUTE_PROPS: VertexAttributeProps = VertexAttributeProps {
    index: 0,
    size: 3,
    stride: 0,
    pointer: std::ptr::null(),
};

fn main() {
    let mut game = Game::new(WindowBuilder {
        width: 1280,
        height: 720,
        title: "Sample Game".to_string(),
    });

    // Initialize game with custom logic
    game.init(|| {
        // Custom initialization logic here
        println!("Custom initialization logic");
    });

    // Create and add a triangle to the game
    let triangle_one_index = game.create_triangle(
        TriangleProps {
            vertices: [-0.3, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0],
        },
        VERTEX_ATTRIBUTE_PROPS,
    );

    fn handle_movement_offset(game: &mut Game) -> (f32, f32) {
        let mut x_offset = 0.0;
        let mut y_offset = 0.0;

        if game.window.input_handler.is_key_pressed(game::KeyInput::W) {
            y_offset += 0.01;
        }

        if game.window.input_handler.is_key_pressed(game::KeyInput::A) {
            x_offset -= 0.01;
        }

        if game.window.input_handler.is_key_pressed(game::KeyInput::S) {
            y_offset -= 0.01;
        }

        if game.window.input_handler.is_key_pressed(game::KeyInput::D) {
            x_offset += 0.01;
        }

        (x_offset, y_offset)
    }

    // Run the game loop with custom update logic
    // Rotate the triangle
    game.run(|game| {
        let elapsed_time = game.elapsed_time;

        // let triangle_props = TriangleProps {
        //     vertices: [
        //         -0.5 * elapsed_time.cos(),
        //         -0.5 * elapsed_time.sin(),
        //         0.0,
        //         0.5 * elapsed_time.cos(),
        //         -0.5 * elapsed_time.sin(),
        //         0.0,
        //         0.0,
        //         0.5 * elapsed_time.cos(),
        //         0.0,
        //     ],
        // };

        // game.update_triangle(triangle_one_index, triangle_props, VERTEX_ATTRIBUTE_PROPS);
    });
}
