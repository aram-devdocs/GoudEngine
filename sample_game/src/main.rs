// sample_app/src/main.rs

use platform::{create_event_loop, App};

fn main() {
    let event_loop = create_event_loop();
    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
