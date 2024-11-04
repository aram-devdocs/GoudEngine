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
use platform::graphics::window::KeyInput as _KeyInput;
use platform::graphics::window::WindowBuilder as _WindowBuilder;
use platform::logger;

// Expose the types from the platform module
pub type VertexAttributeProps = _VertextAttributeProps;
pub type WindowBuilder = _WindowBuilder;
pub type KeyInput = _KeyInput;

// Input props
pub struct TriangleProps {
    pub vertices: [f32; 9],
}

pub struct TriangleData {
    vao: Vao,
    vbo: BufferObject,
}
// Single entry point for the game
pub struct Game {
    pub window: graphics::window::Window,
    pub triangles: Vec<TriangleData>,
    pub elapsed_time: f32,
    // shader_program: Option<ShaderProgram>,
}

impl Game {
    pub fn new(data: WindowBuilder) -> Game {
        logger::init();
        Game {
            window: graphics::window::Window::new(data),
            triangles: vec![],
            elapsed_time: 0.0,
            // triangles should be
            // shader_program: None,
        }
    }

    pub fn init<F>(&mut self, init_callback: F)
    where
        F: FnOnce(),
    {
        self.window.init_gl();
        self.triangles = vec![];
        init_callback();
    }

    pub fn create_triangle(
        &mut self,
        triangle_props: TriangleProps,
        vertex_attribute_props: VertexAttributeProps,
    ) -> u32 {
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new();
        vbo.bind();
        vbo.store_f32_data(&triangle_props.vertices);

        let vertex_attribute = VertexAttribute::new(vertex_attribute_props);
        vertex_attribute.enable();

        vao.unbind();
        vbo.unbind();

        let triangle_data = TriangleData { vao, vbo };

        self.triangles.push(triangle_data);

        let index = self.triangles.len() as u32 - 1;
        index
    }

    pub fn update_triangle(
        &mut self,
        index: u32,
        triangle_props: TriangleProps,
        vertex_attribute_props: VertexAttributeProps,
    ) {
        let triangle_data = &mut self.triangles[index as usize];
        triangle_data.vao.bind();
        triangle_data.vbo.bind();
        triangle_data.vbo.store_f32_data(&triangle_props.vertices);

        let vertex_attribute = VertexAttribute::new(vertex_attribute_props);
        vertex_attribute.enable();

        triangle_data.vao.unbind();
        triangle_data.vbo.unbind();
    }

    pub fn run<F>(&mut self, update_callback: F)
    where
        F: Fn(&mut Game),
    {
        while !self.window.should_close() {
            self.update(&update_callback);
        }

        println!("Window closed!");
    }

    fn update<F>(&mut self, update_callback: &F)
    where
        F: Fn(&mut Game),
    {
        // Update elapsed time
        self.elapsed_time += 0.01;

        // Clear the screen
        clear();

        // Draw the triangle
        draw_arrays(0, 3);

        // Execute custom update logic
        update_callback(self);

        // Update window
        self.window.update();
    }
}
