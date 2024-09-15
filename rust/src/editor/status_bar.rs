use super::{
    terminal::{Size, Terminal},
    view::View,
};

#[derive(Default)]
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

pub struct StatusBar {
    pub document_status: DocumentStatus,
    pub needs_redraw: bool,
    pub width: usize,
    pub margin_bottom: usize,
    pub position_y: usize,
    pub is_visible: bool,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let size = Terminal::size().unwrap_or_default();
        let mut status_bar = Self {
            document_status: DocumentStatus::default(),
            needs_redraw: true,
            width: size.width,
            margin_bottom,
            position_y: 0,
            is_visible: false,
        };
        status_bar.resize(size);
        status_bar
    }

    pub fn resize(&mut self, to: Size) {
        self.width = to.width;

        // if height - mergin_bottom - 1 < 0, then the status bar is not visible
        if let Some(position_y) = to
            .height
            .checked_sub(self.margin_bottom)
            .and_then(|height| height.checked_sub(1))
        {
            self.position_y = position_y;
            self.is_visible = true;
        } else {
            self.position_y = 0;
            self.is_visible = false;
        }
        self.needs_redraw = true;
    }

    pub fn update_status(&mut self, view: &View) {
        let new_status = DocumentStatus {
            filename: format!("{}", view.buffer.file_info).into(),
            total_lines: view.buffer.height(),
            current_line: view.location.y.saturating_add(1),
            modified: view.buffer.dirty > 0,
        };
        self.document_status = new_status;
        self.needs_redraw = true;
    }

    pub fn render(&mut self) {
        if !self.needs_redraw || !self.is_visible {
            return;
        }

        let filename_string = self.document_status.filename_string();
        let modified_string = self.document_status.modified_string();
        let total_lines_string = self.document_status.total_lines_string();

        let left = format!("{filename_string}{modified_string} - {total_lines_string}");
        let right = self.document_status.position_string();
        // minus 1 for the space between left and right
        let reminder_len = self.width.saturating_sub(left.len()).saturating_sub(1);
        let mut line_text = format!("{left} {right:>reminder_len$}");
        line_text.truncate(self.width);
        let result = Terminal::print_invert_row(self.position_y, &line_text);
        debug_assert!(result.is_ok(), "Failed to render status_bar");
        self.needs_redraw = false;
    }
}
