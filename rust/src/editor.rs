use crossterm::event::{
    read,
    Event::{self, Key},
    // Backspace, Delete, Enter,
    KeyEvent,
    KeyEventKind,
};
use terminal::Terminal;
mod editor_command;
mod file_info;
mod terminal;
use editor_command::EditorCommand;
use view::View;
mod view;
use status_bar::StatusBar;
mod status_bar;
use message_bar::MessageBar;
mod message_bar;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    title: String,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        let mut view = View::new(2);
        let args: Vec<String> = std::env::args().collect();
        let status_bar = StatusBar::new(1);
        // only load the first file for now
        if let Some(first) = args.get(1) {
            view.load(first);
        }
        let message_bar = MessageBar::new(0);
        let mut editor = Self {
            should_quit: false,
            view,
            status_bar,
            message_bar,
            title: String::default(),
        };
        editor.refresh_status();
        Ok(editor)
    }

    pub fn refresh_status(&mut self) {
        self.status_bar.update_status(&self.view);
        let filename = self.status_bar.document_status.filename_string();
        let title = format!("{filename} - {NAME}");
        self.message_bar
            .update_message("-------- message bar --------");
        if title != self.title && Terminal::set_title(&title).is_ok() {
            self.title = title;
        }
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
            self.status_bar.update_status(&self.view);
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
            match EditorCommand::try_from(event) {
                Ok(command) => {
                    if matches!(command, EditorCommand::Quit) {
                        self.should_quit = true;
                    } else {
                        self.view.handle_command(command);
                        if let EditorCommand::Resize(size) = command {
                            self.status_bar.resize(size);
                            self.message_bar.resize(size);
                        }
                    }
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    panic!("Could not evaluate command: {err}");
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
        self.status_bar.render();
        self.message_bar.render();
        let _ = Terminal::move_caret_to(self.view.caret_position());
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
