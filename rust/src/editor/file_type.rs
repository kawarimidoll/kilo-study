#[derive(Default, Eq, PartialEq, Debug, Copy, Clone)]
pub enum FileType {
    Rust,
    #[default]
    Text,
}

impl FileType {
    pub fn from(ext: &str) -> Option<Self> {
        match ext {
            "txt" => Some(FileType::Text),
            "rs" => Some(FileType::Rust),
            _ => None,
        }
    }
}
