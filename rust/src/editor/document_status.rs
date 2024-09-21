use super::FileType;
use crate::prelude::LineIdx;

#[derive(Default, Eq, PartialEq)]
pub struct DocumentStatus {
    pub filename: Option<String>,
    pub file_type: Option<FileType>,
    pub total_lines: usize,
    pub current_line_idx: LineIdx,
    pub modified: bool,
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
        format!("{}/{}", self.current_line_idx, self.total_lines)
    }
}
