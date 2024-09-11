use std::{fs::read_to_string, io::Error};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    // pub fn len(&self) -> usize {
    //     self.lines.len()
    // }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    pub fn load(filename: &str) -> Result<Self, Error> {
        let contents = read_to_string(filename)?;
        let mut lines = Vec::new();
        for line in contents.lines() {
            lines.push(String::from(line));
        }
        Ok(Self { lines })
    }
}
