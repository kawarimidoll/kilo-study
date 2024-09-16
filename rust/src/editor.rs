// 日本語🇯🇵の表示テスト
use crossterm::event::{
    read,
    Event::{self, Key},
    // Backspace, Delete, Enter,
    KeyEvent,
    KeyEventKind,
};
use terminal::{Size, Terminal};
mod editor_command;
mod file_info;
mod terminal;
use editor_command::{
    Command::{self, Edit, Move, System},
    System::{Quit, Resize, Save},
};
use view::View;
mod view;
use ui_component::UIComponent;
mod ui_component;
use status_bar::StatusBar;
mod status_bar;
use message_bar::MessageBar;
mod message_bar;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    terminal_size: Size,
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

        let mut editor = Self::default();

        let args: Vec<String> = std::env::args().collect();
        // only load the first file for now
        if let Some(first) = args.get(1) {
            let message = if editor.view.load(first).is_err() {
                &format!("Could not open file: {first}")
            } else {
                "HELP: Ctrl-S = save | Ctrl-Q = quit"
            };

            editor.message_bar.update_message(message);
        }
        let size = Terminal::size().unwrap_or_default();
        editor.resize(size);
        editor.refresh_status();
        Ok(editor)
    }

    pub fn resize(&mut self, size: Size) {
        self.terminal_size = size;
        let view_size = Size {
            width: size.width,
            height: size.height.saturating_sub(2),
        };
        let bar_size = Size {
            width: size.width,
            height: 1,
        };
        self.view.resize(view_size);
        self.status_bar.resize(bar_size);
        self.message_bar.resize(bar_size);
    }

    pub fn refresh_status(&mut self) {
        self.status_bar.update_status(&self.view);
        let filename = self.status_bar.document_status.filename_string();
        let title = format!("{filename} - {NAME}");
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
            match Command::try_from(event) {
                Ok(command) => self.process_command(command),
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
    fn process_command(&mut self, command: Command) {
        match command {
            System(Quit) => self.should_quit = true,
            System(Resize(size)) => self.resize(size),
            System(Save) => self.handle_save(),
            Edit(command) => self.view.handle_edit_command(command),
            Move(command) => self.view.handle_move_command(command),
        }
    }
    fn handle_save(&mut self) {
        let message = if self.view.save().is_ok() {
            "File saved successfully"
        } else {
            "Error saving file"
        };
        self.message_bar.update_message(message);
    }
    fn refresh_screen(&mut self) {
        if self.terminal_size.width == 0 || self.terminal_size.height == 0 {
            return;
        }
        let _ = Terminal::hide_caret();
        let message_origin = self.terminal_size.height.saturating_sub(1);
        self.message_bar.render(message_origin);
        if self.terminal_size.height > 1 {
            let terminal_origin = self.terminal_size.height.saturating_sub(2);
            self.status_bar.render(terminal_origin);
            if self.terminal_size.height > 2 {
                self.view.render(0);
            }
        }

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
