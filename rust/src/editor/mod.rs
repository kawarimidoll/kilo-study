// 日本語🇯🇵の表示テスト
use crate::prelude::*;
use crossterm::event::{
    read,
    Event::{self, Key},
    // Backspace, Delete, Enter,
    KeyEvent,
    KeyEventKind,
};
use terminal::Terminal;
mod command;
mod file_info;
mod terminal;
use command::{
    Command::{self, Edit, Move, System},
    Edit::InsertNewLine,
    Move::{Down, Left, Right, Up},
    System::{Dismiss, Quit, Resize, Save, Search},
};
mod ui_components;
use annotated_string::{AnnotatedString, AnnotationType};
mod annotated_string;
use std::io::Error;
mod line;
use line::Line;
mod document_status;
use document_status::DocumentStatus;
use ui_components::{CommandBar, MessageBar, StatusBar, UIComponent, View};

const QUIT_COUNT: u8 = 2;

#[derive(Default, Eq, PartialEq)]
enum PromptType {
    Save,
    Search,
    #[default]
    None,
}
impl PromptType {
    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
    terminal_size: Size,
    title: String,
    quit_count: u8,
    prompt_type: PromptType,
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
            debug_assert!(!first.is_empty());
            let message = if editor.view.load(first).is_err() {
                &format!("Could not open file: {first}")
            } else {
                "HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit"
            };

            editor.message_bar.update_message(message);
        }
        let size = Terminal::size().unwrap_or_default();
        editor.handle_resize_command(size);
        editor.refresh_status();
        Ok(editor)
    }

    pub fn handle_resize_command(&mut self, size: Size) {
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
        self.command_bar.resize(bar_size);
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
                    #[cfg(not(debug_assertions))]
                    let _ = err;
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
        if let System(Resize(size)) = command {
            self.handle_resize_command(size);
            return;
        }
        match self.prompt_type {
            PromptType::Search => self.process_command_during_search(command),
            PromptType::Save => self.process_command_during_save(command),
            PromptType::None => self.process_command_during_no_prompt(command),
        }
    }
    fn process_command_during_no_prompt(&mut self, command: Command) {
        if matches!(command, System(Quit)) {
            self.handle_quit();
            return;
        }
        self.reset_quit_count();
        // handle edit / move commands or start save / search
        match command {
            System(Save) => {
                if self.view.buffer.file_info.has_path() {
                    self.save(None);
                } else {
                    self.show_prompt(PromptType::Save);
                }
            }
            System(Search) => self.show_prompt(PromptType::Search),
            Edit(command) => self.view.handle_edit_command(command),
            Move(command) => self.view.handle_move_command(command),
            System(_) => {}
        }
    }
    fn process_command_during_save(&mut self, command: Command) {
        match command {
            System(Dismiss) => {
                self.show_prompt(PromptType::None);
                self.message_bar.update_message("Aborted.");
            }
            Edit(InsertNewLine) => {
                self.save(Some(&self.command_bar.value()));
                // it can't set None here because of multiple mutable borrows
                self.show_prompt(PromptType::None);
            }
            Edit(command) => self.command_bar.handle_edit_command(command),
            _ => {}
        }
    }
    fn process_command_during_search(&mut self, command: Command) {
        match command {
            System(Dismiss) => {
                self.show_prompt(PromptType::None);
                self.view.dismiss_search();
                self.message_bar.update_message("Aborted.");
                self.message_bar.set_needs_redraw(true);
            }
            Edit(InsertNewLine) => {
                // it can't set None here because of multiple mutable borrows
                self.show_prompt(PromptType::None);
                self.view.exit_search();
            }
            Edit(command) => {
                self.command_bar.handle_edit_command(command);
                self.view.search(&self.command_bar.value());
            }
            Move(Down | Right) => self.view.search_next(),
            Move(Up | Left) => self.view.search_prev(),
            _ => {}
        }
    }
    fn handle_quit(&mut self) {
        if self.view.buffer.dirty == 0 || self.quit_count == 0 {
            self.should_quit = true;
        } else if self.view.buffer.dirty > 0 {
            self.message_bar.update_message(&format!(
                "Unsaved changes. Press Ctrl-Q {} more times to quit.",
                self.quit_count,
            ));
            self.quit_count = self.quit_count.saturating_sub(1);
        }
    }
    fn reset_quit_count(&mut self) {
        if self.quit_count < QUIT_COUNT {
            self.quit_count = QUIT_COUNT;
            self.message_bar.update_message("");
        }
    }
    fn show_prompt(&mut self, prompt_type: PromptType) {
        match prompt_type {
            PromptType::Save => self.command_bar.set_prompt("Save as: "),
            PromptType::Search => {
                self.view.enter_search();
                self.command_bar.set_prompt("Search: ");
            }
            PromptType::None => self.message_bar.set_needs_redraw(true),
        }

        self.command_bar.clear_value();
        self.prompt_type = prompt_type;
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
        if self.prompt_type.is_none() {
            self.message_bar.render(bottom_row);
        } else {
            self.command_bar.render(bottom_row);
        }
        if self.terminal_size.height > 1 {
            let terminal_origin = self.terminal_size.height.saturating_sub(2);
            self.status_bar.render(terminal_origin);
            if self.terminal_size.height > 2 {
                self.view.render(0);
            }
        }

        let caret_position = if self.prompt_type.is_none() {
            self.view.caret_position()
        } else {
            Position {
                col: self.command_bar.caret_col(),
                row: bottom_row,
            }
        };
        debug_assert!(caret_position.col < self.terminal_size.width);
        debug_assert!(caret_position.row < self.terminal_size.height);

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
