use crate::libs::platform;
use crate::libs::platform::ecs::ECS;
use crate::libs::platform::graphics::rendering::clear;
use crate::libs::platform::graphics::rendering::renderer2d::Renderer2D;
use crate::libs::platform::graphics::rendering::Renderer;

use crate::types::SpriteCreateDto;
use crate::types::TextureManager;
use crate::types::TiledManager;
use platform::logger;

pub use platform::graphics::window::Window;
pub use platform::graphics::window::WindowBuilder;

/// Single entry point for the game
#[repr(C)]
pub struct GameSdk {
    pub window: Window,
    pub renderer_2d: Option<Renderer2D>,
    pub elapsed_time: f32,
    pub ecs: ECS,
    pub texture_manager: TextureManager,
    pub tiled_manager: TiledManager,
    pub tiled_map_sprite_ids: Option<Vec<u32>>,
    pub new_tileset: bool,
}

impl GameSdk {
    pub fn new(data: WindowBuilder) -> GameSdk {
        logger::init();
        let window = platform::graphics::window::Window::new(data);

        GameSdk {
            window,
            renderer_2d: None,
            elapsed_time: 0.0,
            ecs: ECS::new(),
            texture_manager: TextureManager::new(),
            tiled_manager: TiledManager::new(),
            tiled_map_sprite_ids: None,
            new_tileset: false,
        }
    }

    pub extern "C" fn init<F>(&mut self, init_callback: F)
    where
        F: FnOnce(&mut GameSdk),
    {
        self.window.init_gl();
        let window_width = self.window.width;
        let window_height = self.window.height;
        self.renderer_2d = Some(
            Renderer2D::new(window_width, window_height).expect("Failed to create Renderer2D"),
        );
        init_callback(self);
    }

    pub extern "C" fn start<F>(&mut self, start_callback: F)
    where
        F: FnOnce(&mut GameSdk),
    {
        start_callback(self);
    }

    pub extern "C" fn update<F>(&mut self, update_callback: &F)
    where
        F: Fn(&mut GameSdk),
    {
        self.elapsed_time += 0.01;
        clear();

        update_callback(self);

        // Manage tiled maps
        if self.new_tileset {
            let texture_manager = &self.texture_manager;
            self.new_tileset = false;
            if let Some(selected_map_id) = self.tiled_manager.selected_map_id {
                if let Some(tiled) = self.tiled_manager.get_map_by_id(selected_map_id) {
                    //

                    let tile_layers =
                        tiled
                            .map
                            .layers()
                            .filter_map(|layer| match layer.layer_type() {
                                tiled::LayerType::Tiles(layer) => Some(layer),
                                _ => None,
                            });

                    let tilesets = tiled.map.tilesets();

                    // remove all entities associated with the current map befopre adding new ones
                    if let Some(sprite_ids) = &self.tiled_map_sprite_ids {
                        for sprite_id in sprite_ids {
                            if let Err(e) = self.ecs.remove_sprite(*sprite_id) {
                                eprintln!("Failed to remove sprite: {}", e);
                            }
                        }
                    }

                    for layer in tile_layers {
                        let height = layer.height();
                        let width = layer.width();

                        // height and width are both option u32. If they are None, we return an error. if they are Some, we get the value.
                        let height = match height {
                            Some(h) => h,
                            None => {
                                eprintln!("Height not found");
                                continue;
                            }
                        };
                        let width = match width {
                            Some(w) => w,
                            None => {
                                eprintln!("Width not found");
                                continue;
                            }
                        };

                        for y in 0..height {
                            for x in 0..width {
                                let tile = layer.get_tile(x as i32, y as i32);

                                // TODO: Create from tile
                                let data: SpriteCreateDto = SpriteCreateDto {
                                    x: x as f32 * 32.0,
                                    y: y as f32 * 32.0,
                                    z_layer: 0,
                                    scale_x: 1.0,
                                    scale_y: 1.0,
                                    dimension_x: 32.0,
                                    dimension_y: 32.0,
                                    rotation: 0.0,
                                    source_rect: crate::types::Rectangle {
                                        x: 0.0,
                                        y: 0.0,
                                        width: 32.0,
                                        height: 32.0,
                                    },
                                    texture_id: 0,
                                    debug: false,
                                    frame: crate::types::Rectangle {
                                        x: 0.0,
                                        y: 0.0,
                                        width: 32.0,
                                        height: 32.0,
                                    },
                                };

                                let sprite_id = self.ecs.add_sprite(data);

                                // add sprite id to tiled_map_sprite_ids
                                if let Some(sprite_ids) = &mut self.tiled_map_sprite_ids {
                                    sprite_ids.push(sprite_id);
                                } else {
                                    self.tiled_map_sprite_ids = Some(vec![sprite_id]);
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Some(renderer) = &mut self.renderer_2d {
            renderer.render(self.ecs.sprites.clone(), &self.texture_manager);
        }

        self.window.update();
    }
    pub extern "C" fn terminate(&mut self) {
        self.window.terminate();
        self.ecs.terminate();
        if let Some(renderer) = &mut self.renderer_2d {
            renderer.terminate();
        }
    }

    // pub fn should_close(&self) -> bool {
    //     self.window.should_close()
    // }
}
