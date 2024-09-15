use std::{
    fmt::{self, Display},
    path::PathBuf,
};

#[derive(Default, Debug)]
pub struct FileInfo {
    pub path: Option<PathBuf>,
}

impl FileInfo {
    pub fn from(filename: &str) -> Self {
        Self {
            path: Some(PathBuf::from(filename)),
        }
    }
}

impl Display for FileInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = self
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("[No Name]");
        write!(formatter, "{name}",)
    }
}
