use crossterm::event::{
    read,
    Event::{self, Key},
    KeyCode::{self, Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    // Backspace, Delete, Enter,
    KeyEvent,
    KeyEventKind,
    KeyModifiers,
};
use terminal::{Position, Size, Terminal};
mod terminal;
use core::cmp::min;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Copy, Clone, Default)]
pub struct Location {
    // the position of the document
    pub x: usize,
    pub y: usize,
}
impl Location {
    pub fn as_potition(&self) -> Position {
        Position {
            col: self.x,
            row: self.y,
        }
    }
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    message: usize,
    location: Location,
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
            self.evaluate_event(&event)?;
        }
        Ok(())
    }
    fn evaluate_event(&mut self, event: &Event) -> Result<(), Error> {
        if let Key(KeyEvent {
            code,
            modifiers,
            // necessary for windows
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            match code {
                Char('q') if *modifiers == KeyModifiers::CONTROL => self.should_quit = true,

                Left | Down | Right | Up | Home | End | PageDown | PageUp => {
                    self.move_point(*code)?;
                }
                // Delete => self.message = 9,
                // Backspace => self.message = 10,
                // Enter => self.message = 11,
                _ => (),
            }
        }
        Ok(())
    }
    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_caret()?;
        Terminal::move_caret_to(Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye!\r\n")?;
        } else {
            self.draw_rows()?;
            Terminal::move_caret_to(self.location.as_potition())?;
        }
        Terminal::show_caret()?;
        Terminal::execute()?;
        Ok(())
    }
    fn move_point(&mut self, code: KeyCode) -> Result<(), Error> {
        let Location { x, y } = self.location;
        let Size { width, height } = Terminal::size()?;
        let max_x = width.saturating_sub(1);
        let max_y = height.saturating_sub(1);
        match code {
            Left => self.location.x = x.saturating_sub(1),
            Right => self.location.x = min(max_x, x.saturating_add(1)),
            Up => self.location.y = y.saturating_sub(1),
            Down => self.location.y = min(max_y, y.saturating_add(1)),
            Home => self.location.x = 0,
            End => self.location.x = max_x,
            PageUp => self.location.y = 0,
            PageDown => self.location.y = max_y,
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
