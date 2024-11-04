use game::Game;
use platform::graphics::gl_wrapper::{Vao, BufferObject, ShaderProgram};

fn main() {
    let mut game = Game::new(1280, 720, "Sample Game");

    // Initialize OpenGL
    game.window.init_gl();

    // Define vertices for a triangle
    let vertices: [f32; 9] = [
        0.0,  0.5, 0.0,  // Vertex 1 (X, Y)
       -0.5, -0.5, 0.0,  // Vertex 2 (X, Y)
        0.5, -0.5, 0.0   // Vertex 3 (X, Y)
    ];

    // Create and bind VAO
    let vao = Vao::new();
    vao.bind();


    // Create and bind VBO
    let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
    vbo.bind();
    vbo.store_f32_data(&vertices);

    // Define vertex attributes
    let position_attribute = platform::graphics::gl_wrapper::VertexAttribute::new(
        0, 3, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<f32>() as i32, std::ptr::null()
    );
    position_attribute.enable();

    // Load and compile shaders
    // let shader_program = ShaderProgram::new(
    //     "path/to/vertex_shader.glsl",
    //     "path/to/fragment_shader.glsl"
    // );
    // shader_program.bind();

    // Main loop
    while !game.window.should_close() {
        // Clear the screen
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Draw the triangle
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        // Update window
        game.window.update();
    }

    game.run();
}
