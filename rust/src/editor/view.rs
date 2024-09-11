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
    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
        Ok(())
    }
    pub fn render_welcome_screen(&self) -> Result<(), Error> {
        for current_row in 0..self.size.height.saturating_sub(1) {
            Terminal::move_caret_to(Position {
                col: 0,
                row: current_row,
            })?;
            Terminal::clear_line()?;
            Self::draw_empty_row()?;
        }
        self.draw_welcome_message()?;
        Ok(())
    }
    pub fn render_buffer(&self) -> Result<(), Error> {
        for current_row in 0..self.size.height.saturating_sub(1) {
            Terminal::move_caret_to(Position {
                col: 0,
                row: current_row,
            })?;
            Terminal::clear_line()?;
            if let Some(line) = self.buffer.lines.get(current_row) {
                let mut l = String::from(line);
                l.truncate(self.size.width);
                Terminal::print(&l)?;
            } else {
                Self::draw_empty_row()?;
            }
        }
        Ok(())
    }
    pub fn render(&mut self) -> Result<(), Error> {
        // render function
        if !self.needs_redraw || self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }
        if self.buffer.is_empty() {
            self.render_welcome_screen()?;
        } else {
            self.render_buffer()?;
        }
        Terminal::move_caret_to(Position {
            col: 0,
            row: self.size.height.saturating_sub(1),
        })?;
        // here comes status line
        Terminal::print("--------")?;
        self.needs_redraw = false;
        Ok(())
    }
}
