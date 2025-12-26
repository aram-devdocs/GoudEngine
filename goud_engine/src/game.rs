use crate::libs::ecs::Ecs;
use crate::libs::graphics::clear;
use crate::libs::graphics::{
    renderer::RendererType, renderer2d::Renderer2D, renderer3d::Renderer3D,
};

use crate::libs::logger;
use crate::types::Rectangle;
use crate::types::SpriteCreateDto;
use crate::types::TextureManager;
use crate::types::TiledManager;

use crate::libs::platform::window::{Window, WindowBuilder};

/// Single entry point for the game
#[repr(C)]
pub struct GameSdk {
    pub window: Window,
    pub renderer: Option<RendererType>,
    pub elapsed_time: f32,
    pub ecs: Ecs,
    pub texture_manager: TextureManager,
    pub tiled_manager: TiledManager,
    pub tiled_map_sprite_ids: Option<Vec<u32>>,
    pub new_tileset: bool,
    renderer_type: i32,
}

impl GameSdk {
    pub fn new(data: WindowBuilder, renderer_type: i32) -> GameSdk {
        logger::init();
        let window = Window::new(data);

        GameSdk {
            window,
            renderer: None,
            elapsed_time: 0.0,
            ecs: Ecs::new(),
            texture_manager: TextureManager::new(),
            tiled_manager: TiledManager::new(),
            tiled_map_sprite_ids: None,
            new_tileset: false,
            renderer_type,
        }
    }

    pub extern "C" fn init<F>(&mut self, init_callback: F)
    where
        F: FnOnce(&mut GameSdk),
    {
        self.window.init_gl();
        let window_width = self.window.width;
        let window_height = self.window.height;

        // Initialize renderer based on type
        self.renderer = match self.renderer_type {
            0 => Some(RendererType::new_2d(
                Renderer2D::new(window_width, window_height).expect("Failed to create Renderer2D"),
            )),
            1 => Some(RendererType::new_3d(
                Renderer3D::new(window_width, window_height).expect("Failed to create Renderer3D"),
            )),
            _ => panic!("Invalid renderer type"),
        };

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
        // TODO: this is elapsed updates, not elapsed time
        self.elapsed_time += 0.01;
        clear();

        update_callback(self);

        // Manage tiled maps
        self.manage_tileset();

        if let Some(renderer) = &mut self.renderer {
            renderer.render(&self.ecs.sprites, &self.texture_manager);
        }

        self.window.update();
    }

    pub extern "C" fn terminate(&mut self) {
        self.window.terminate();
        self.ecs.terminate();
        if let Some(renderer) = &mut self.renderer {
            renderer.terminate();
        }
    }

    // TODO: this should be moved to libs/graphics/tiled
    /// Helper function to manage the tileset
    fn manage_tileset(&mut self) {
        if !self.new_tileset {
            return;
        }

        self.new_tileset = false;

        let selected_map_id = match self.tiled_manager.selected_map_id {
            Some(id) => id,
            None => return,
        };

        let tiled = match self.tiled_manager.get_map_by_id(selected_map_id) {
            Some(tiled) => tiled,
            None => return,
        };

        // Remove existing sprites
        if let Some(sprite_ids) = &self.tiled_map_sprite_ids {
            for sprite_id in sprite_ids {
                let _ = self.ecs.remove_sprite(*sprite_id);
            }
            self.tiled_map_sprite_ids = None;
        }

        let mut sprite_ids = Vec::new(); // Collect new sprite ids

        let tile_layers = tiled
            .map
            .layers()
            .filter_map(|layer| match layer.layer_type() {
                tiled::LayerType::Tiles(layer) => Some(layer),
                _ => None,
            });

        let mut layer_index = 0;

        for layer in tile_layers {
            if let (Some(height), Some(width)) = (layer.height(), layer.width()) {
                for y in 0..height {
                    for x in 0..width {
                        if let Some(tile) = layer.get_tile(x as i32, y as i32) {
                            let tileset = tile.get_tileset();
                            let tile_height = tileset.tile_height;
                            let tile_width = tileset.tile_width;
                            let tile_id = tile.id();

                            let data = SpriteCreateDto {
                                x: x as f32 * tile_width as f32,
                                y: y as f32 * tile_height as f32,
                                z_layer: layer_index,
                                scale_x: if tile.flip_h { -1.0 } else { 1.0 },
                                scale_y: if tile.flip_v { -1.0 } else { 1.0 },
                                dimension_x: tile_width as f32,
                                dimension_y: tile_height as f32,
                                rotation: 0.0,
                                source_rect: Rectangle {
                                    x: 0.0,
                                    y: 0.0,
                                    width: tile_width as f32,
                                    height: tile_height as f32,
                                },
                                texture_id: tiled.texture_ids[tile.tileset_index()],
                                debug: false,
                                frame: Rectangle {
                                    x: tileset.tile_width as f32
                                        * (tile_id % tileset.columns) as f32,
                                    y: tileset.tile_height as f32
                                        * (tile_id / tileset.columns) as f32,
                                    width: tileset.tile_width as f32,
                                    height: tileset.tile_height as f32,
                                },
                            };

                            let sprite_id = self.ecs.add_sprite(data);
                            sprite_ids.push(sprite_id);
                        }
                    }
                }
            } else {
                eprintln!("Layer dimensions not found");
                continue;
            }
            // Increment layer_index after processing each layer
            layer_index += 1;
        }

        self.tiled_map_sprite_ids = Some(sprite_ids);
    }

    // pub fn should_close(&self) -> bool {
    //     self.window.should_close()
    // }
}
