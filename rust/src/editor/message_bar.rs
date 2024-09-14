use super::terminal::{Size, Terminal};

pub struct MessageBar {
    pub message: String,
    pub needs_redraw: bool,
    pub width: usize,
    pub margin_bottom: usize,
    pub position_y: usize,
}

impl MessageBar {
    pub fn new(margin_bottom: usize) -> Self {
        let Size { width, height } = Terminal::size().unwrap_or_default();
        Self {
            message: String::default(),
            needs_redraw: true,
            width,
            margin_bottom,
            position_y: height.saturating_sub(margin_bottom).saturating_sub(1),
        }
    }

    pub fn resize(&mut self, to: Size) {
        self.width = to.width;
        self.position_y = to
            .height
            .saturating_sub(self.margin_bottom)
            .saturating_sub(1);
        self.needs_redraw = true;
    }

    pub fn update_message(&mut self, message: &str) {
        self.message = String::from(message);
        self.needs_redraw = true;
    }
    // pub fn clear_message(&mut self) {
    //     self.update_message("");
    // }

    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }

        let mut line_text = self.message.clone();
        line_text.truncate(self.width);
        let result = Terminal::print_row(self.position_y, &line_text);
        debug_assert!(result.is_ok(), "Failed to render status_bar");
        self.needs_redraw = false;
    }
}
