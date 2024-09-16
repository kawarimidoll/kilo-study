use super::{line::Line, location::Location};
use std::fs::{read_to_string, File};
use std::io::{Error, Write};

use crate::editor::file_info::FileInfo;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_info: FileInfo,
    pub dirty: usize,
}

impl Buffer {
    pub fn height(&self) -> usize {
        self.lines.len()
    }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    pub fn insert_newline(&mut self, at: Location) -> bool {
        let Location { x, y } = at;
        if y >= self.height() {
            self.lines.push(Line::default());
        } else {
            // we have a valid y
            let second_half = self.lines[y].split_off(x);
            self.lines.insert(y.saturating_add(1), second_half);
        }
        self.dirty = self.dirty.saturating_add(1);
        true
    }
    pub fn remove_char(&mut self, at: Location) -> bool {
        let Location { x, y } = at;
        // out of bounds
        if y >= self.height() {
            return false;
        }

        // below here, we have a valid y
        if x < self.lines[y].len() {
            self.lines[y].remove(x, 1);
        } else if y < self.height().saturating_sub(1) {
            let next_line = self.lines.remove(y.saturating_add(1));
            self.lines[y].append(&next_line);
        } else {
            // the last line, the last character
            return false;
        }
        self.dirty = self.dirty.saturating_add(1);
        true
    }
    pub fn insert_char(&mut self, c: char, at: Location) -> bool {
        let Location { x, y } = at;
        // out of bounds
        if y > self.height() {
            return false;
        }

        let string = c.to_string();

        // append a new line
        if y == self.height() {
            self.lines.push(Line::from(&string));
            self.dirty = self.dirty.saturating_add(1);
            return true;
        }

        // insert a new character in an existing line
        if let Some(line) = self.lines.get_mut(y) {
            line.insert(x, &string);
            self.dirty = self.dirty.saturating_add(1);
            return true;
        }

        // maybe dead code, but the compiler doesn't know that
        false
    }
    pub fn load(filename: &str) -> Result<Self, Error> {
        let contents = read_to_string(filename)?;
        let mut lines = Vec::new();
        for line in contents.lines() {
            lines.push(Line::from(line));
        }
        Ok(Self {
            lines,
            file_info: FileInfo::from(filename),
            dirty: 0,
        })
    }
    pub fn save(&mut self) -> Result<(), Error> {
        self.save_to_file()
    }
    pub fn save_as(&mut self, filename: &str) -> Result<(), Error> {
        let file_info = FileInfo::from(filename);
        self.file_info = file_info;
        self.save_to_file()
    }
    pub fn save_to_file(&mut self) -> Result<(), Error> {
        if let Some(path) = &self.file_info.get_path() {
            let mut file = File::create(path)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
            self.dirty = 0;
        }
        Ok(())
    }
}
