// 日本語🇯🇵の表示テスト
use crossterm::event::{
    read,
    Event::{self, Key},
    // Backspace, Delete, Enter,
    KeyEvent,
    KeyEventKind,
};
use terminal::{Position, Size, Terminal};
mod editor_command;
mod file_info;
mod terminal;
use editor_command::{
    Command::{self, Edit, Move, System},
    Edit::InsertNewLine,
    System::{Dismiss, Quit, Resize, Save},
};
use view::View;
mod view;
use ui_component::UIComponent;
mod ui_component;
use status_bar::StatusBar;
mod status_bar;
use message_bar::MessageBar;
mod message_bar;
use command_bar::CommandBar;
mod command_bar;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUIT_COUNT: u8 = 2;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: Option<CommandBar>,
    terminal_size: Size,
    title: String,
    quit_count: u8,
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
        editor.reset_quit_count();

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
        if let Some(command_bar) = &mut self.command_bar {
            command_bar.resize(bar_size);
        }
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
            System(Quit) => {
                if self.command_bar.is_none() {
                    self.handle_quit();
                }
            }
            System(Resize(size)) => self.resize(size),
            _ => self.reset_quit_count(),
        }
        match command {
            // already handled above
            System(Quit) | System(Resize(_)) => {}
            System(Save) => {
                if self.command_bar.is_none() {
                    self.handle_save();
                }
            }
            System(Dismiss) => {
                if self.command_bar.is_some() {
                    self.dismiss_prompt();
                    self.message_bar.update_message("Save aborted.");
                    self.message_bar.set_needs_redraw(true);
                }
            }
            Edit(command) => {
                if let Some(command_bar) = &mut self.command_bar {
                    if matches!(command, InsertNewLine) {
                        let filename = command_bar.value();
                        // it can't set None here because of multiple mutable borrows
                        self.dismiss_prompt();
                        self.save(Some(&filename));
                    } else {
                        command_bar.handle_edit_command(command);
                    }
                } else {
                    self.view.handle_edit_command(command)
                }
            }
            Move(command) => self.view.handle_move_command(command),
        }
    }
    fn dismiss_prompt(&mut self) {
        self.command_bar = None;
        self.message_bar.set_needs_redraw(true);
    }
    fn handle_quit(&mut self) {
        if self.view.buffer.dirty == 0 || self.quit_count == 0 {
            self.should_quit = true;
        } else if self.view.buffer.dirty > 0 {
            self.message_bar.update_message(&format!(
                "Unsaved changes. Press Ctrl-Q {} more times to quit.",
                self.quit_count,
            ));
            self.quit_count -= 1;
        }
    }
    fn reset_quit_count(&mut self) {
        if self.quit_count < QUIT_COUNT {
            self.quit_count = QUIT_COUNT;
            self.message_bar.update_message("");
        }
    }
    fn show_prompt(&mut self) {
        let mut command_bar = CommandBar::default();
        command_bar.set_prompt("Save as: ");
        let bar_size = Size {
            width: self.terminal_size.width,
            height: 1,
        };
        command_bar.set_size(bar_size);
        command_bar.set_needs_redraw(true);
        self.command_bar = Some(command_bar);
    }
    fn handle_save(&mut self) {
        if self.view.buffer.file_info.path.is_none() {
            self.show_prompt();
        } else {
            self.save(None);
        }
    }
    fn save(&mut self, filename: Option<&str>) {
        let save_result = if let Some(name) = filename {
            self.view.save_as(name)
        } else {
            self.view.save()
        };

        let message = if save_result.is_ok() {
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
        let bottom_row = self.terminal_size.height.saturating_sub(1);
        if let Some(command_bar) = &mut self.command_bar {
            command_bar.render(bottom_row);
        } else {
            self.message_bar.render(bottom_row);
        }
        if self.terminal_size.height > 1 {
            let terminal_origin = self.terminal_size.height.saturating_sub(2);
            self.status_bar.render(terminal_origin);
            if self.terminal_size.height > 2 {
                self.view.render(0);
            }
        }

        let caret_position = if let Some(command_bar) = &self.command_bar {
            Position {
                col: command_bar.caret_col(),
                row: bottom_row,
            }
        } else {
            self.view.caret_position()
        };
        let _ = Terminal::move_caret_to(caret_position);
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
