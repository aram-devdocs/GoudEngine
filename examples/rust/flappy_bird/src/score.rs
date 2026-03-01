use crate::constants::*;
use crate::engine::Engine;

/// Tracks the player's score and renders it as digit sprites.
pub struct ScoreCounter {
    pub score: u32,
    digit_textures: [u64; 10],
    x_offset: f32,
    y_offset: f32,
}

impl ScoreCounter {
    pub fn new() -> Self {
        Self {
            score: 0,
            digit_textures: [0; 10],
            x_offset: SCREEN_WIDTH / 2.0 - 30.0,
            y_offset: 50.0,
        }
    }

    /// Loads digit textures 0-9 from the shared assets directory.
    pub fn load_textures(&mut self, engine: &Engine, asset_base: &str) {
        for i in 0..10 {
            self.digit_textures[i] = engine.load_texture(&format!("{asset_base}/sprites/{i}.png"));
        }
    }

    pub fn increment(&mut self) {
        self.score += 1;
    }

    pub fn reset(&mut self) {
        self.score = 0;
    }

    /// Draws the score at the top-center of the screen.
    pub fn draw(&self, engine: &Engine) {
        let score_str = self.score.to_string();
        for (i, ch) in score_str.chars().enumerate() {
            let digit = (ch as u32 - '0' as u32) as usize;
            let x = self.x_offset + i as f32 * SCORE_DIGIT_SPACING + SCORE_DIGIT_WIDTH / 2.0;
            let y = self.y_offset + SCORE_DIGIT_HEIGHT / 2.0;
            engine.draw_sprite(
                self.digit_textures[digit],
                x,
                y,
                SCORE_DIGIT_WIDTH,
                SCORE_DIGIT_HEIGHT,
                0.0,
            );
        }
    }
}
