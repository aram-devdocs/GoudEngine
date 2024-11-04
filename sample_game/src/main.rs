use game::{Game, TriangleProps, VertexAttributeProps, WindowBuilder};

const VERTEX_ATTRIBUTE_PROPS: VertexAttributeProps = VertexAttributeProps {
    index: 0,
    size: 3,
    stride: 0,
    pointer: std::ptr::null(),
};

const INITIAL_PLAYER_ONE_VERTEX: [f32; 9] = [-0.2, -0.2, 0.0, 0.2, -0.2, 0.0, 0.0, 0.2, 0.0];

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
            vertices: INITIAL_PLAYER_ONE_VERTEX,
            position: Some((0.0, 0.0)),
            rotation: Some(0.0),
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

    fn handle_rotation_offset(game: &mut Game) -> f32 {
        let mut rotation_offset = 0.0;

        if game.window.input_handler.is_key_pressed(game::KeyInput::Q) {
            rotation_offset += 0.01;
        }

        if game.window.input_handler.is_key_pressed(game::KeyInput::E) {
            rotation_offset -= 0.01;
        }

        rotation_offset
    }

    // Run the game loop with custom update logic
    // Rotate the triangle and move it around the screen
    game.run(|game| {
        // handle close on escape
        if game
            .window
            .input_handler
            .is_key_pressed(game::KeyInput::Escape)
        {
            game.window.close_window();
        }

        let (x_offset, y_offset) = handle_movement_offset(game);
        let current_position = game.get_triangle_position(triangle_one_index);
        let new_position = (current_position.0 + x_offset, current_position.1 + y_offset);

        let current_rotation = game.get_triangle_rotation(triangle_one_index);
        let rotation_offset = handle_rotation_offset(game);
        let new_rotation = current_rotation + rotation_offset;

        let mut vertices = game.get_triangle_vertices(triangle_one_index);

        // use the rotation to rotate the triangle before applying the position
        let sin = new_rotation.sin();
        let cos = new_rotation.cos();

        for i in (0..vertices.len()).step_by(3) {
            let x = vertices[i];
            let y = vertices[i + 1];

            vertices[i] = x * cos - y * sin;
            vertices[i + 1] = x * sin + y * cos;
        }

        // apply the position
        vertices[0] += new_position.0;
        vertices[1] += new_position.1;
        vertices[3] += new_position.0;
        vertices[4] += new_position.1;
        vertices[6] += new_position.0;
        vertices[7] += new_position.1;

        game.update_triangle(
            triangle_one_index,
            TriangleProps {
                position: Some(new_position),
                rotation: Some(new_rotation),
                vertices,
            },
            VERTEX_ATTRIBUTE_PROPS,
        );

        // game.update_triangle(triangle_one_index, triangle_props, VERTEX_ATTRIBUTE_PROPS);
    });
}
