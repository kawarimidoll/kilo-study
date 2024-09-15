use super::{
    terminal::{Size, Terminal},
    view::View,
};
use crossterm::style::Attribute;

#[derive(Default)]
pub struct DocumentStatus {
    filename: Option<String>,
    total_lines: usize,
    current_line: usize,
    modified: bool,
}
impl DocumentStatus {
    pub fn filename_string(&self) -> String {
        let filename = self
            .filename
            .clone()
            .unwrap_or_else(|| String::from("[No Name]"));
        let modified = if self.modified { "(modified)" } else { "" };
        return format!("{filename}{modified}");
    }
    pub fn total_lines_string(&self) -> String {
        return format!("{} lines", self.total_lines);
    }
    pub fn position_string(&self) -> String {
        return format!("{}/{}", self.current_line, self.total_lines);
    }
}

pub struct StatusBar {
    pub document_status: DocumentStatus,
    pub needs_redraw: bool,
    pub width: usize,
    pub margin_bottom: usize,
    pub position_y: usize,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let Size { width, height } = Terminal::size().unwrap_or_default();
        Self {
            document_status: DocumentStatus::default(),
            needs_redraw: true,
            width,
            margin_bottom,
            position_y: height.saturating_sub(margin_bottom).saturating_sub(1),
        }
    }

    pub fn resize(&mut self, to: Size) {
        self.width = to.width;
        self.position_y = to
            .height
            .saturating_sub(self.margin_bottom)
            .saturating_sub(1);
        self.needs_redraw = true;
    }

    pub fn update_status(&mut self, view: &View) {
        let new_status = DocumentStatus {
            filename: view.buffer.filename.clone(),
            total_lines: view.buffer.height(),
            current_line: view.location.y.saturating_add(1),
            modified: view.buffer.dirty > 0,
        };
        self.document_status = new_status;
        self.needs_redraw = true;
    }

    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }

        let filename_string = self.document_status.filename_string();
        let total_lines_string = self.document_status.total_lines_string();

        let left = format!("{filename_string} - {total_lines_string}");
        let right = self.document_status.position_string();
        let padding_len = self
            .width
            .saturating_sub(left.len())
            .saturating_sub(right.len());
        let padding = " ".repeat(padding_len);
        let mut line_text = format!("{left}{padding}{right}");
        line_text.truncate(self.width);
        let result = Terminal::print_row(
            self.position_y,
            &format!("{}{line_text}{}", Attribute::Reverse, Attribute::Reset),
        );
        debug_assert!(result.is_ok(), "Failed to render status_bar");
        self.needs_redraw = false;
    }
}
