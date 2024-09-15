use super::terminal::{Size, Terminal};

#[derive(Default)]
pub struct MessageBar {
    pub message: String,
    pub needs_redraw: bool,
    pub width: usize,
}

impl MessageBar {
    pub fn resize(&mut self, to: Size) {
        self.width = to.width;
        self.needs_redraw = true;
    }

    pub fn update_message(&mut self, message: String) {
        if self.message != message {
            self.message = message;
            self.needs_redraw = true;
        }
    }

    pub fn render(&mut self, position_y: usize) {
        if !self.needs_redraw {
            return;
        }

        let mut line_text = self.message.clone();
        line_text.truncate(self.width);
        let result = Terminal::print_row(position_y, &line_text);
        debug_assert!(result.is_ok(), "Failed to render status_bar");
        self.needs_redraw = false;
    }
}
