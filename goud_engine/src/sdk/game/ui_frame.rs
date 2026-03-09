//! Per-frame UI processing hooks for [`GoudGame`](super::GoudGame).

use super::GoudGame;

impl GoudGame {
    /// Processes UI layout + input for one frame using the game context window size.
    pub(crate) fn process_ui_frame(&mut self) {
        let viewport = self.context.window_size();
        self.ui_manager.set_viewport_size(viewport.0, viewport.1);

        #[cfg(feature = "native")]
        {
            self.ui_manager.process_input_frame(&mut self.input_manager);
        }

        #[cfg(not(feature = "native"))]
        {
            self.ui_manager.update();
        }
    }

    /// Processes UI layout + input for one frame using an explicit viewport.
    #[cfg(feature = "native")]
    pub(crate) fn process_ui_frame_with_viewport(&mut self, viewport: (u32, u32)) {
        self.ui_manager.set_viewport_size(viewport.0, viewport.1);
        self.ui_manager.process_input_frame(&mut self.input_manager);
    }
}
