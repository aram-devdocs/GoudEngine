use game::Game;

fn main() {
    let game = Game::new();
    let event_loop = (game.create_event_loop)();
    let mut app = game.app;
    app.draw_polygon();
    let _ = event_loop.run_app(&mut app);
}
