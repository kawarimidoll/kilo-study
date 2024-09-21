use crate::prelude::{RowIdx, Size};

use super::super::{DocumentStatus, Terminal};
use super::{UIComponent, View};
use std::io::Error;

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
            file_type: view.buffer.file_info.get_file_type(),
            total_lines: view.buffer.height(),
            current_line_idx: view.text_location.line_idx.saturating_add(1),
            modified: view.buffer.dirty > 0,
        };
        if self.document_status != new_status {
            self.document_status = new_status;
            self.set_needs_redraw(true);
        }
    }
}

impl UIComponent for StatusBar {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    fn set_size(&mut self, to: Size) {
        self.width = to.width;
    }
    fn draw(&mut self, origin_row: RowIdx) -> Result<(), Error> {
        let filename_string = self.document_status.filename_string();
        let modified_string = self.document_status.modified_string();
        let total_lines_string = self.document_status.total_lines_string();
        let position_string = self.document_status.position_string();
        let file_type_string = self.document_status.file_type.as_ref().map_or_else(
            String::default,
            |file_type| format!("{file_type:?} | " ),
        );

        let left = format!("{filename_string}{modified_string} - {total_lines_string}");
        let right = format!("{file_type_string}{position_string}");
        // minus 1 for the space between left and right
        let reminder_len = self.width.saturating_sub(left.len()).saturating_sub(1);
        let mut line_text = format!("{left} {right:>reminder_len$}");
        line_text.truncate(self.width);
        let result = Terminal::print_invert_row(origin_row, &line_text);
        debug_assert!(result.is_ok(), "Failed to render status_bar");
        Ok(())
    }
}
