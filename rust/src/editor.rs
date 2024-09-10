use crossterm::event::{
    read,
    Event::{self, Key},
    KeyCode::{Backspace, Char, Delete, Down, End, Enter, Home, Left, PageDown, PageUp, Right, Up},
    KeyEvent, KeyModifiers,
};
use terminal::{Position, Terminal};
mod terminal;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    message: usize,
}

impl Editor {
    pub const fn default() -> Self {
        Self {
            should_quit: false,
            message: 0,
        }
    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(&event);
        }
        Ok(())
    }
    fn evaluate_event(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => self.should_quit = true,

                Left => self.message = 1,
                Down => self.message = 2,
                Right => self.message = 3,
                Up => self.message = 4,
                Home => self.message = 5,
                End => self.message = 6,
                PageDown => self.message = 7,
                PageUp => self.message = 8,
                Delete => self.message = 9,
                Backspace => self.message = 10,
                Enter => self.message = 11,

                _ => (),
            }
        }
    }
    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye!\r\n")?;
        } else {
            self.draw_rows()?;
            Terminal::move_cursor_to(Position { x: 0, y: 0 })?;
        }
        Terminal::show_cursor()?;
        Terminal::execute()?;
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
        Terminal::print(format!("{message}\r"))?;
        Ok(())
    }
    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
        Ok(())
    }
    fn draw_rows(&self) -> Result<(), Error> {
        let height = Terminal::size()?.height;
        for current_row in 0..height.saturating_add(1) {
            Terminal::clear_line()?;
            // we alow this since we don't care if our welcome message is put *exactly* in the middle.
            // it's allowed to be a bit up or down
            #[allow(clippy::integer_division)]
            if current_row == height / 3 {
                Self::draw_welcome_message()?;
            }
            Self::draw_empty_row()?;
            // to ensure it works, add `.`
            Terminal::print(".\r\n")?;
        }
        // Self::draw_empty_row()?;
        Terminal::print(format!("{0}", self.message))?;
        Ok(())
    }
}
