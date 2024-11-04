// use platform::custom_errors;
use platform::graphics;
use platform::graphics::gl_wrapper::VertexAttributeProps;
use platform::graphics::gl_wrapper::{BufferObject, ShaderProgram, Vao};
use platform::graphics::window::WindowBuilder;
use platform::logger;

pub struct Game {
    pub window: graphics::window::Window,
    vao: Option<Vao>,
    vbo: Option<BufferObject>,
    shader_program: Option<ShaderProgram>,
}

pub struct TriangleProps {
    pub vertices: [f32; 9],
}

// pub struct VertexAttributeProps {
//     pub index: u32,
//     pub size: i32,
//     pub r#type: GLenum,
//     pub normalized: GLboolean,
//     pub stride: GLsizei,
//     pub pointer: *const c_void,
// }

impl Game {
    pub fn new(data: WindowBuilder) -> Game {
        logger::init();
        Game {
            window: graphics::window::Window::new(data),
            vao: None,
            vbo: None,
            shader_program: None,
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
        // Define vertices for a triangle
        let vertices = triangle_props.vertices;

        // Create and bind VAO
        let vao = Vao::new();
        vao.bind();
        self.vao = Some(vao);

        // Create and bind VBO
        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(&vertices);
        self.vbo = Some(vbo);

        // Define vertex attributes
        let position_attribute = graphics::gl_wrapper::VertexAttribute::new(vertex_attribute_props);
        position_attribute.enable();

        // Load and compile shaders (commented out as shaders are not provided)
        // let shader_program = ShaderProgram::new(
        //     "path/to/vertex_shader.glsl",
        //     "path/to/fragment_shader.glsl"
        // );
        // shader_program.bind();
        // self.shader_program = Some(shader_program);
    }

    pub fn run(&mut self) {
        while !self.window.should_close() {
            self.update();
        }

        println!("Window closed!");
    }

    fn update(&mut self) {
        // Clear the screen
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Draw the triangle
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        // Update window
        self.window.update();
    }
}
