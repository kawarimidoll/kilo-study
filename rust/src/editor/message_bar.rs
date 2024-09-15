use std::io::Error;
use super::terminal::{Size, Terminal};
use super::ui_component::UIComponent;

#[derive(Default)]
pub struct MessageBar {
    pub message: String,
    pub needs_redraw: bool,
    pub width: usize,
}

impl MessageBar {
    pub fn update_message(&mut self, message: String) {
        if self.message != message {
            self.message = message;
            self.needs_redraw = true;
        }
    }
}

impl UIComponent for MessageBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    fn set_size(&mut self, to: Size) {
        self.width = to.width;
    }
    fn draw(&mut self, origin_y: usize) -> Result<(), Error> {
        let mut line_text = self.message.clone();
        line_text.truncate(self.width);
        let result = Terminal::print_row(origin_y, &line_text);
        debug_assert!(result.is_ok(), "Failed to render status_bar");
        Ok(())
    }
}
