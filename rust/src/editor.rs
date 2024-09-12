use crossterm::event::{
    read,
    Event::{self, Key},
    KeyCode::{Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    // Backspace, Delete, Enter,
    KeyEvent,
    KeyEventKind,
    KeyModifiers,
};
use terminal::{Size, Terminal};
mod terminal;
use view::View;
mod view;
use std::io::Error;

pub struct Editor {
    should_quit: bool,
    view: View,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        let mut view = View::default();
        let args: Vec<String> = std::env::args().collect();
        // only load the first file for now
        if let Some(first) = args.get(1) {
            view.load(first);
        }
        Ok(Self {
            should_quit: false,
            view,
        })
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(event) => self.evaluate_event(event),

                Err(err) => {
                    #[cfg(debug_assertions)]
                    panic!("Could not read event: {err}");
                }
            }
        }
    }

    // needless_pass_by_value: Event is not huge, so there is not a performance overhead in passing
    // by value, and pattern matching on it is more ergonomic.
    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        let should_process = match event {
            Key(KeyEvent { kind, .. }) => kind == KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };
        if should_process {
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
                        self.view.move_point(code);
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
                _ => {
                    #[cfg(debug_assertions)]
                    panic!("Could not handle command");
                }
            }
        } else {
            #[cfg(debug_assertions)]
            panic!("Received and discarded unsupported or non-press event.");
        }
    }
    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        self.view.render();
        let _ = Terminal::move_caret_to(self.view.location.as_potition());
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
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
