use super::terminal::{Size, Terminal};
use super::ui_component::UIComponent;
use std::{
    io::Error,
    time::{Duration, Instant},
};
const DEFAULT_DURATION: Duration = Duration::new(5, 0);

struct Message {
    text: String,
    time: Instant,
    duration: Duration,
}
impl Default for Message {
    fn default() -> Self {
        Self::new(String::new())
    }
}
impl Message {
    fn new(text: String) -> Self {
        Self {
            text,
            time: Instant::now(),
            duration: DEFAULT_DURATION,
        }
    }
    fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.time) > self.duration
    }
}

#[derive(Default)]
pub struct MessageBar {
    message: Message,
    needs_redraw: bool,
    width: usize,
    cleared_after_expiry: bool,
}

impl MessageBar {
    pub fn update_message(&mut self, new_message: &str) {
        self.message = Message::new(new_message.to_string());
        self.cleared_after_expiry = false;
        self.set_needs_redraw(true);
    }
}

impl UIComponent for MessageBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    fn needs_redraw(&self) -> bool {
        self.needs_redraw || (!self.cleared_after_expiry && self.message.is_expired())
    }
    fn set_size(&mut self, to: Size) {
        self.width = to.width;
    }
    fn draw(&mut self, origin_y: usize) -> Result<(), Error> {
        let line_text = if self.message.is_expired() {
            // write blank string to clear the message bar
            //
            self.cleared_after_expiry = true;
            ""
        } else {
            &self.message.text
        };
        let result = Terminal::print_row(origin_y, line_text);
        debug_assert!(result.is_ok(), "Failed to render status_bar");
        Ok(())
    }
}
