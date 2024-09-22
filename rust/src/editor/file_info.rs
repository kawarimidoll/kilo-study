use super::FileType;
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Default, Debug)]
pub struct FileInfo {
    path: Option<PathBuf>,
    file_type: Option<FileType>,
}

impl FileInfo {
    pub fn from(filename: &str) -> Self {
        let path = Some(PathBuf::from(filename));

        let file_type = path
            .as_ref()
            .and_then(|path| path.extension())
            .and_then(|ext| ext.to_str())
            .and_then(FileType::from);
        Self { path, file_type }
    }
    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }
    pub fn extension(&self) -> Option<&str> {
        self.path
            .as_ref()
            .and_then(|path| path.extension())
            .and_then(|ext| ext.to_str())
    }
    pub fn get_file_type(&self) -> Option<FileType> {
        self.file_type
    }
}

impl fmt::Display for FileInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = self
            .get_path()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("[No Name]");
        write!(formatter, "{name}",)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_type() {
        assert_eq!(
            FileInfo::from("sample.txt").get_file_type().unwrap(),
            FileType::Text
        );
        assert_eq!(
            FileInfo::from("sample.rs").get_file_type().unwrap(),
            FileType::Rust
        );
        assert_eq!(FileInfo::from("sample.unknown").get_file_type(), None);
    }
    #[test]
    fn test_extension() {
        assert_eq!(FileInfo::from("sample.txt").extension().unwrap(), "txt");
        assert_eq!(FileInfo::from("sample.rs").extension().unwrap(), "rs");
        assert_eq!(FileInfo::from("no_ext").extension(), None);
    }
}
