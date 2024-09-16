use super::{
    terminal::{Size, Terminal},
    ui_component::UIComponent,
    view::View,
};
use std::io::Error;

#[derive(Default, Eq, PartialEq)]
pub struct DocumentStatus {
    filename: Option<String>,
    total_lines: usize,
    current_line: usize,
    modified: bool,
}
impl DocumentStatus {
    pub fn filename_string(&self) -> String {
        self.filename
            .clone()
            .unwrap_or_else(|| String::from("[No Name]"))
    }
    pub fn modified_string(&self) -> String {
        if self.modified {
            String::from("(modified)")
        } else {
            String::default()
        }
    }
    pub fn total_lines_string(&self) -> String {
        format!("{} lines", self.total_lines)
    }
    pub fn position_string(&self) -> String {
        format!("{}/{}", self.current_line, self.total_lines)
    }
}

#[derive(Default)]
pub struct StatusBar {
    pub document_status: DocumentStatus,
    pub needs_redraw: bool,
    pub width: usize,
}

impl StatusBar {
    pub fn update_status(&mut self, view: &View) {
        let new_status = DocumentStatus {
            filename: format!("{}", view.buffer.file_info).into(),
            total_lines: view.buffer.height(),
            current_line: view.location.y.saturating_add(1),
            modified: view.buffer.dirty > 0,
        };
        if self.document_status != new_status {
            self.document_status = new_status;
            self.mark_redraw(true);
        }
    }
}

impl UIComponent for StatusBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    fn set_size(&mut self, to: Size) {
        self.width = to.width;
    }
    fn draw(&mut self, origin_y: usize) -> Result<(), Error> {
        let filename_string = self.document_status.filename_string();
        let modified_string = self.document_status.modified_string();
        let total_lines_string = self.document_status.total_lines_string();

        let left = format!("{filename_string}{modified_string} - {total_lines_string}");
        let right = self.document_status.position_string();
        // minus 1 for the space between left and right
        let reminder_len = self.width.saturating_sub(left.len()).saturating_sub(1);
        let mut line_text = format!("{left} {right:>reminder_len$}");
        line_text.truncate(self.width);
        let result = Terminal::print_invert_row(origin_y, &line_text);
        debug_assert!(result.is_ok(), "Failed to render status_bar");
        Ok(())
    }
}
