use super::super::{command::Edit, Terminal};
use super::UIComponent;
use crate::prelude::{ColIdx, RowIdx, Size};
use std::cmp::min;
use std::io::Error;

#[derive(Default)]
pub struct CommandBar {
    prompt: String,
    value: String,
    needs_redraw: bool,
    size: Size,
}

impl CommandBar {
    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(c) => self.insert(c),
            Edit::DeleteBackward => self.delete_backward(),
            Edit::Delete | Edit::InsertNewLine => {}
        }
        self.set_needs_redraw(true);
    }
    fn insert(&mut self, c: char) {
        self.value.push(c);
    }
    fn delete_backward(&mut self) {
        self.value.pop();
    }
    pub fn caret_col(&self) -> ColIdx {
        let max_width = self.prompt.len().saturating_add(self.value.len());
        min(max_width, self.size.width)
    }
    pub fn value(&self) -> String {
        self.value.to_string()
    }
    pub fn clear_value(&mut self) {
        self.value = String::default();
    }
    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_string();
        self.set_needs_redraw(true);
    }
}

impl UIComponent for CommandBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    fn set_size(&mut self, to: Size) {
        self.size = to;
    }
    fn draw(&mut self, origin_row: RowIdx) -> Result<(), Error> {
        let area_width_for_value = self.size.width.saturating_sub(self.prompt.len());
        let value_end = self.value.len();
        let value_start = value_end.saturating_sub(area_width_for_value);

        let line_text = format!("{}{}", self.prompt, &self.value[value_start..]);
        let result = Terminal::print_row(origin_row, &line_text);
        debug_assert!(result.is_ok(), "Failed to render command_bar");
        Ok(())
    }
}
