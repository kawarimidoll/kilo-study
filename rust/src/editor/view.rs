use buffer::Buffer;
mod buffer;
use super::terminal::{Position, Size, Terminal};
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    pub buffer: Buffer,
    pub needs_redraw: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
        }
    }
}

impl View {
    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }
    fn draw_welcome_message() -> Result<(), Error> {
        let width = Terminal::size()?.width;

        let title = format!("{NAME} editor -- version {VERSION}");
        // we alow this since we don't care if our welcome message is put *exactly* in the middle.
        // it's allowed to be a bit to the left or right.
        #[allow(clippy::integer_division)]
        let padding = " ".repeat(width.saturating_sub(title.len()) / 2);
        let mut message = format!("{padding}{title}");
        message.truncate(width);
        Terminal::print(&format!("{message}\r"))?;
        Ok(())
    }
    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
        Ok(())
    }
    pub fn render_welcome_screen() -> Result<(), Error> {
        let height = Terminal::size()?.height;
        for current_row in 0..height.saturating_sub(1) {
            Terminal::move_caret_to(Position {
                col: 0,
                row: current_row,
            })?;
            Terminal::clear_line()?;
            // we alow this since we don't care if our welcome message is put *exactly* in the middle.
            // it's allowed to be a bit up or down
            #[allow(clippy::integer_division)]
            if current_row == height / 3 {
                Self::draw_welcome_message()?;
            }
            Self::draw_empty_row()?;
        }
        Ok(())
    }
    pub fn render_buffer(&self) -> Result<(), Error> {
        let Size { width, height } = Terminal::size()?;
        for current_row in 0..height.saturating_sub(1) {
            Terminal::move_caret_to(Position {
                col: 0,
                row: current_row,
            })?;
            Terminal::clear_line()?;
            if let Some(line) = self.buffer.lines.get(current_row) {
                let mut l = String::from(line);
                l.truncate(width);
                Terminal::print(&l)?;
            } else {
                Self::draw_empty_row()?;
            }
        }
        Ok(())
    }
    pub fn render(&mut self) -> Result<(), Error> {
        // render function
        if !self.needs_redraw {
            return Ok(());
        }
        if self.buffer.is_empty() {
            Self::render_welcome_screen()?;
        } else {
            self.render_buffer()?;
        }
        Terminal::move_caret_to(Position {
            col: 0,
            row: Terminal::size()?.height.saturating_sub(1),
        })?;
        // here comes status line
        Terminal::print("--------")?;
        self.needs_redraw = false;
        Ok(())
    }
}
