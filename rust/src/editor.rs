use crossterm::event::{
    read,
    Event::{self, Key},
    KeyCode::{self, Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    // Backspace, Delete, Enter,
    KeyEvent,
    KeyModifiers,
};
use terminal::{Position, Terminal};
mod terminal;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    message: usize,
    position: Position,
}

impl Editor {
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

                Left | Down | Right | Up | Home | End | PageDown | PageUp => {
                    self.move_position(*code);
                }
                // Delete => self.message = 9,
                // Backspace => self.message = 10,
                // Enter => self.message = 11,
                _ => (),
            }
        }
    }
    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_caret()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::move_caret_to(Position::default())?;
            Terminal::print("Goodbye!\r\n")?;
        } else {
            self.draw_rows()?;
            Terminal::move_caret_to(self.position)?;
        }
        Terminal::show_caret()?;
        Terminal::execute()?;
        Ok(())
    }
    fn move_position(&mut self, code: KeyCode) -> Result<(), Error> {
        match code {
            Left if self.position.x > 0 => self.position.x -= 1,
            Right if self.position.x < Terminal::size()?.width => self.position.x += 1,
            Up if self.position.y > 0 => self.position.y -= 1,
            Down if self.position.y < Terminal::size()?.height => self.position.y += 1,
            Home => self.position.x = 0,
            End => self.position.x = Terminal::size()?.width,
            PageUp => self.position.y = 0,
            PageDown => self.position.y = Terminal::size()?.height,
            _ => (),
        };
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
