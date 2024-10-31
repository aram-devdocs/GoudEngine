use crate::renderer::Renderer;
use crate::gui::Gui;

pub struct Engine {
    renderer: Renderer,
    gui: Gui,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            renderer: Renderer::new(),
            gui: Gui::new(),
        }
    }

    pub fn run(&mut self) {
        println!("Running engine...");

        loop {
            // Placeholder game loop
            self.update();
            self.render();

            // Break the loop conditionally (e.g., on a quit event)
            break;
        }
    }

    fn update(&mut self) {
        // Update game state here
        println!("Updating engine...");
    }

    fn render(&mut self) {
        // Call the renderer to handle drawing
        self.renderer.render();
        self.gui.draw();
    }
}