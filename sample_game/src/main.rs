use game::Game;

fn main() {
    let game = Game::new();
    let event_loop = (game.create_event_loop)();
    let mut app = game.app;
    let _ = event_loop.run_app(&mut app);
}
