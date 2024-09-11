use buffer::Buffer;
mod buffer;
use super::terminal::{Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    pub buffer: Buffer,
    pub needs_redraw: bool,
    pub size: Size,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
        }
    }
}

impl View {
    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.needs_redraw = true;
    }
    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }
    fn draw_welcome_message(&self) {
        let messages = vec![
            "Welcome!".to_string(),
            format!("{NAME} editor -- version {VERSION}"),
        ];

        // we don't care if our welcome message is put *exactly* in the middle.
        #[allow(clippy::integer_division)]
        let mut row = self.size.height / 3;
        for mut message in messages {
            #[allow(clippy::integer_division)]
            let col = self.size.width.saturating_sub(message.len()) / 2;
            message = format!("~{}{}", " ".repeat(col), message);
            message.truncate(self.size.width.saturating_sub(1));
            Self::render_line(row, &message);
            row = row.saturating_add(1);
        }
    }
    fn render_line(row: usize, line_text: &str) {
        let result = Terminal::print_row(row, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }
    pub fn render_buffer(&self) {
        for current_row in 0..self.size.height.saturating_sub(1) {
            if let Some(line) = self.buffer.lines.get(current_row) {
                let mut l = String::from(line);
                l.truncate(self.size.width);
                Self::render_line(current_row, &l);
            } else {
                Self::render_line(current_row, "~");
            }
        }
    }
    pub fn render(&mut self) {
        // render function
        if !self.needs_redraw || self.size.width == 0 || self.size.height == 0 {
            return;
        }
        self.render_buffer();
        if self.buffer.is_empty() {
            self.draw_welcome_message();
        }
        // here comes status line
        Self::render_line(self.size.height.saturating_sub(1), "--------");
        self.needs_redraw = false;
    }
}
