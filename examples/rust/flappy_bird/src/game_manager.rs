use crate::bird::Bird;
use crate::constants::*;
use crate::engine::{Engine, KEY_ESCAPE, KEY_R, KEY_SPACE, MOUSE_LEFT};
use crate::pipe::Pipe;
use crate::score::ScoreCounter;

/// Loaded texture handles (created once at init, reused every frame).
pub struct Textures {
    pub background: u64,
    pub base: u64,
    pub pipe: u64,
    pub bird_frames: [u64; 3],
}

impl Textures {
    pub fn load(engine: &Engine, asset_base: &str) -> Self {
        let background = engine.load_texture(&format!("{asset_base}/sprites/background-day.png"));
        let base = engine.load_texture(&format!("{asset_base}/sprites/base.png"));
        let pipe = engine.load_texture(&format!("{asset_base}/sprites/pipe-green.png"));
        let bird_frames = [
            engine.load_texture(&format!("{asset_base}/sprites/bluebird-downflap.png")),
            engine.load_texture(&format!("{asset_base}/sprites/bluebird-midflap.png")),
            engine.load_texture(&format!("{asset_base}/sprites/bluebird-upflap.png")),
        ];
        Self {
            background,
            base,
            pipe,
            bird_frames,
        }
    }
}

/// Orchestrates game state: bird, pipes, scoring, collision, rendering.
pub struct GameManager {
    bird: Bird,
    pipes: Vec<Pipe>,
    score: ScoreCounter,
    textures: Textures,
    pipe_spawn_timer: f32,
}

impl GameManager {
    pub fn new(engine: &Engine, asset_base: &str) -> Self {
        let textures = Textures::load(engine, asset_base);
        let mut score = ScoreCounter::new();
        score.load_textures(engine, asset_base);
        Self {
            bird: Bird::new(),
            pipes: Vec::new(),
            score,
            textures,
            pipe_spawn_timer: 0.0,
        }
    }

    /// Resets all game state for a fresh round.
    pub fn start(&mut self) {
        self.bird.reset();
        self.pipes.clear();
        self.score.reset();
        self.pipe_spawn_timer = 0.0;
    }

    /// Runs one frame of game logic. Returns `false` when the window
    /// should close (user pressed Escape).
    pub fn update(&mut self, engine: &Engine, dt: f32) -> bool {
        // -- Input --------------------------------------------------------
        if engine.is_key_pressed(KEY_ESCAPE) {
            return false;
        }
        if engine.is_key_pressed(KEY_R) {
            self.start();
            return true;
        }

        let jump = engine.is_key_pressed(KEY_SPACE) || engine.is_mouse_button_pressed(MOUSE_LEFT);

        // -- Update bird --------------------------------------------------
        self.bird.update(dt, jump);

        // Ground / ceiling collision
        if self.bird.y + BIRD_HEIGHT > SCREEN_HEIGHT || self.bird.y < 0.0 {
            self.start();
            return true;
        }

        // -- Update pipes and check collisions ----------------------------
        for pipe in &mut self.pipes {
            pipe.update(dt);
            if pipe.collides_with_bird(self.bird.x, self.bird.y, BIRD_WIDTH, BIRD_HEIGHT) {
                self.start();
                return true;
            }
        }

        // -- Spawn new pipes ----------------------------------------------
        self.pipe_spawn_timer += dt;
        if self.pipe_spawn_timer > PIPE_SPAWN_INTERVAL {
            self.pipe_spawn_timer = 0.0;
            self.pipes.push(Pipe::new());
        }

        // -- Remove off-screen pipes and increment score ------------------
        let before = self.pipes.len();
        self.pipes.retain(|pipe| !pipe.is_off_screen());
        let removed = before - self.pipes.len();
        for _ in 0..removed {
            self.score.increment();
        }

        true
    }

    /// Draws all layers in painter's order (back to front).
    pub fn draw(&self, engine: &Engine) {
        let tex = &self.textures;

        // Layer 0: Background
        engine.draw_sprite(
            tex.background,
            BACKGROUND_WIDTH / 2.0,
            BACKGROUND_HEIGHT / 2.0,
            BACKGROUND_WIDTH,
            BACKGROUND_HEIGHT,
            0.0,
        );

        // Layer 1: Score (behind pipes, in front of background)
        self.score.draw(engine);

        // Layer 2: Pipes
        for pipe in &self.pipes {
            // Top pipe (rotated 180 degrees)
            engine.draw_sprite(
                tex.pipe,
                pipe.x + PIPE_IMAGE_WIDTH / 2.0,
                pipe.top_pipe_y() + PIPE_IMAGE_HEIGHT / 2.0,
                PIPE_IMAGE_WIDTH,
                PIPE_IMAGE_HEIGHT,
                std::f32::consts::PI,
            );
            // Bottom pipe (no rotation)
            engine.draw_sprite(
                tex.pipe,
                pipe.x + PIPE_IMAGE_WIDTH / 2.0,
                pipe.bottom_pipe_y() + PIPE_IMAGE_HEIGHT / 2.0,
                PIPE_IMAGE_WIDTH,
                PIPE_IMAGE_HEIGHT,
                0.0,
            );
        }

        // Layer 3: Bird
        let bird_tex = tex.bird_frames[self.bird.animator.frame_index];
        let rotation_rad = self.bird.rotation_deg().to_radians();
        engine.draw_sprite(
            bird_tex,
            self.bird.x + BIRD_WIDTH / 2.0,
            self.bird.y + BIRD_HEIGHT / 2.0,
            BIRD_WIDTH,
            BIRD_HEIGHT,
            rotation_rad,
        );

        // Layer 4: Base / ground (in front of everything)
        engine.draw_sprite(
            tex.base,
            BASE_SPRITE_WIDTH / 2.0,
            SCREEN_HEIGHT + BASE_HEIGHT / 2.0,
            BASE_SPRITE_WIDTH,
            BASE_HEIGHT,
            0.0,
        );
    }
}
