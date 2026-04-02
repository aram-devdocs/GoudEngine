use goudengine::*;

fn main() {
    let engine = Engine::new(800, 600, "My Game");
    engine.enable_blending();

    while !engine.should_close() {
        let dt = engine.poll_events();
        engine.begin_frame();
        engine.clear(0.2, 0.2, 0.2, 1.0);
        // game logic here
        engine.end_frame();
        engine.swap_buffers();
    }
}
