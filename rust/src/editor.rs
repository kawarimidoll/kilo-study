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
use view::View;
mod view;
use core::cmp::min;
use std::io::Error;

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
    location: Location,
    view: View,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        self.handle_args();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    pub fn handle_args(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        // only load the first file for now
        if let Some(first) = args.get(1) {
            self.view.load(first);
        }
    }

    fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(event);
        }
        Ok(())
    }

    // needless_pass_by_value: Event is not huge, so there is not a performance overhead in passing
    // by value, and pattern matching on it is more ergonomic.
    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        match event {
            Key(KeyEvent {
                code,
                modifiers,
                // necessary for windows
                kind: KeyEventKind::Press,
                ..
            }) => match (code, modifiers) {
                (Char('q'), KeyModifiers::CONTROL) => self.should_quit = true,

                (Left | Down | Right | Up | Home | End | PageDown | PageUp, _) => {
                    self.move_point(code);
                }
                _ => (),
            },
            Event::Resize(width16, height16) => {
                #[allow(clippy::as_conversions)]
                let width = width16 as usize;
                #[allow(clippy::as_conversions)]
                let height = height16 as usize;
                self.view.resize(Size { width, height });
            }
            _ => (),
        }
    }
    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        let _ = Terminal::move_caret_to(Position::default());
        self.view.render();
        let _ = Terminal::move_caret_to(self.location.as_potition());
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
    fn move_point(&mut self, code: KeyCode) {
        let Location { x, y } = self.location;
        let Size { width, height } = Terminal::size().unwrap_or_default();
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
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye!\r\n");
        }
    }
}
