// use platform::custom_errors;
use platform::graphics;
use platform::graphics::gl_wrapper::{
    clear,
    draw_arrays,
    BufferObject,
    //  ShaderProgram,
    Vao,
    VertexAttribute,
    VertexAttributeProps as _VertextAttributeProps,
};
use platform::graphics::window::WindowBuilder as _WindowBuilder;
use platform::logger;

// Expose the types from the platform module
pub type VertexAttributeProps = _VertextAttributeProps;
pub type WindowBuilder = _WindowBuilder;

// Input props
pub struct TriangleProps {
    pub vertices: [f32; 9],
}

// Single entry point for the game
pub struct Game {
    pub window: graphics::window::Window,
    // shader_program: Option<ShaderProgram>,
}

impl Game {
    pub fn new(data: WindowBuilder) -> Game {
        logger::init();
        Game {
            window: graphics::window::Window::new(data),
            // shader_program: None,
        }
    }

    pub fn init<F>(&mut self, init_callback: F)
    where
        F: FnOnce(),
    {
        self.window.init_gl();
        init_callback();
    }

    pub fn create_triangle(
        &mut self,
        triangle_props: TriangleProps,
        vertex_attribute_props: VertexAttributeProps,
    ) {
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new();
        vbo.bind();
        vbo.store_f32_data(&triangle_props.vertices);

        let vertex_attribute = VertexAttribute::new(vertex_attribute_props);
        vertex_attribute.enable();

        vao.unbind();
        vbo.unbind();
    }

    pub fn run<F>(&mut self, update_callback: F)
    where
        F: Fn(),
    {
        while !self.window.should_close() {
            self.update(&update_callback);
        }

        println!("Window closed!");
    }

    fn update<F>(&mut self, update_callback: &F)
    where
        F: Fn(),
    {
        // Clear the screen
        clear();

        // Draw the triangle
        draw_arrays(0, 3);

        // Execute custom update logic
        update_callback();

        // Update window
        self.window.update();
    }
}
