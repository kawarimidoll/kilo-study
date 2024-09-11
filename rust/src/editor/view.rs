use buffer::Buffer;
mod buffer;
use super::terminal::{Position, Size, Terminal};
use std::io::Error;

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
    fn draw_welcome_message(&self) -> Result<(), Error> {
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
            message.truncate(self.size.width.saturating_sub(col));
            Terminal::move_caret_to(Position { col, row })?;
            Terminal::print(&message)?;
            row = row.saturating_add(1);
        }

        Ok(())
    }
    fn render_line(row: usize, line_text: &str) -> Result<(), Error> {
        Terminal::move_caret_to(Position { col: 0, row })?;
        Terminal::clear_line()?;
        Terminal::print(line_text)?;
        Ok(())
    }
    pub fn render_buffer(&self) -> Result<(), Error> {
        for current_row in 0..self.size.height.saturating_sub(1) {
            if let Some(line) = self.buffer.lines.get(current_row) {
                let mut l = String::from(line);
                l.truncate(self.size.width);
                Self::render_line(current_row, &l)?;
            } else {
                Self::render_line(current_row, "~")?;
            }
        }
        Ok(())
    }
    pub fn render(&mut self) -> Result<(), Error> {
        // render function
        if !self.needs_redraw || self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }
        self.render_buffer()?;
        if self.buffer.is_empty() {
            self.draw_welcome_message()?;
        }
        // here comes status line
        Self::render_line(self.size.height.saturating_sub(1), "--------")?;
        self.needs_redraw = false;
        Ok(())
    }
}
