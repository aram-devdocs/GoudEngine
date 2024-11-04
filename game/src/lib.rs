// use platform::custom_errors;
use platform::graphics;
use platform::graphics::gl_wrapper::{
    clear,
    draw_arrays,
    BufferObject,
    //  ShaderProgram,
    Vao,
};
use platform::graphics::gl_wrapper::{
    VertexAttribute, VertexAttributeProps as _VertextAttributeProps,
};
use platform::graphics::window::WindowBuilder as _WindowBuilder;
use platform::logger;

pub struct Game {
    pub window: graphics::window::Window,
    // shader_program: Option<ShaderProgram>,
}

// Expose the types from the platform module
pub type VertexAttributeProps = _VertextAttributeProps;
pub type WindowBuilder = _WindowBuilder;
pub struct TriangleProps {
    pub vertices: [f32; 9],
}

impl Game {
    pub fn new(data: WindowBuilder) -> Game {
        logger::init();
        Game {
            window: graphics::window::Window::new(data),
            // shader_program: None,
        }
    }

    pub fn init(&mut self) {
        self.window.init_gl();
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
    pub fn run(&mut self) {
        while !self.window.should_close() {
            self.update();
        }

        println!("Window closed!");
    }

    fn update(&mut self) {
        // Clear the screen

        clear();

        // // Draw the triangle
        draw_arrays(0, 3);

        // Update window
        self.window.update();
    }
}
