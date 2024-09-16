use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
};

#[derive(Default, Debug)]
pub struct FileInfo {
    path: Option<PathBuf>,
}

impl FileInfo {
    pub fn from(filename: &str) -> Self {
        Self {
            path: Some(PathBuf::from(filename)),
        }
    }
    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }
}

impl Display for FileInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = self
            .get_path()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("[No Name]");
        write!(formatter, "{name}",)
    }
}
