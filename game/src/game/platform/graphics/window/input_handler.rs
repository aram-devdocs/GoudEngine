use glfw::{Action, Key, WindowEvent};
use std::collections::HashSet;

#[repr(C)]
pub struct InputHandler {
    keys_pressed: HashSet<Key>,
}
#[no_mangle]
pub type KeyInput = glfw::Key;
impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {
            keys_pressed: HashSet::new(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Key(key, _, action, _) => match action {
                Action::Press => {
                    self.keys_pressed.insert(*key);
                }
                Action::Release => {
                    self.keys_pressed.remove(key);
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn is_key_pressed(&self, key: KeyInput) -> bool {
        self.keys_pressed.contains(&key)
    }
}
