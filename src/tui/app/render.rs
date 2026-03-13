//! UI rendering for TUI application

use ratatui::Frame;

use crate::tui::screens::get_renderer;

use super::TuiApp;

impl TuiApp {
    /// Draw the UI - routes to current screen renderer
    pub(crate) fn draw(&mut self, frame: &mut Frame) {
        // Draw current screen content (NavMenuWidget is rendered by each screen)
        let renderer = get_renderer(self.current_screen);
        renderer.render(frame, &self.state);

        // Draw toast notifications on top (no clone overhead)
        self.toast_manager
            .render_ref(frame.area(), frame.buffer_mut());
    }
}
