use buffer::Buffer;
mod buffer;
use super::terminal::Terminal;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct View {
    pub buffer: Buffer,
}

impl View {
    pub fn load(&mut self, filename: String) -> Result<(), Error> {
        self.buffer.load(filename)?;
        Ok(())
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
    pub fn render(&self) -> Result<(), Error> {
        // render function
        let height = Terminal::size()?.height;
        for current_row in 0..height.saturating_sub(1) {
            Terminal::clear_line()?;
            if let Some(line) = self.buffer.lines.get(current_row) {
                Terminal::print(line)?;
                Terminal::print("\r\n")?;
                continue;
            }
            // we alow this since we don't care if our welcome message is put *exactly* in the middle.
            // it's allowed to be a bit up or down
            #[allow(clippy::integer_division)]
            if current_row == height / 3 && self.buffer.is_empty() {
                Self::draw_welcome_message()?;
            }
            Self::draw_empty_row()?;
            // to ensure it works, add `.`
            Terminal::print(".\r\n")?;
        }
        // here comes status line
        Terminal::print("--------")?;
        Ok(())
    }
}
