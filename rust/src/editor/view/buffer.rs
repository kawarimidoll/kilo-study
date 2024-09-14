use super::{line::Line, location::Location};
use std::{fs::read_to_string, io::Error};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn height(&self) -> usize {
        self.lines.len()
    }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    pub fn remove_char(&mut self, at: Location) -> bool {
        let Location { x, y } = at;
        // out of bounds
        if x == 0 && y == 0 {
            return false;
        }
        if x == 0 {
            // todo: join with previous line
            return false;
        }

        if let Some(line) = self.lines.get_mut(y) {
            line.remove(x, 1);
            return true;
        }

        false
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
            return true;
        }

        // insert a new character in an existing line
        if let Some(line) = self.lines.get_mut(y) {
            line.insert(x, &string);
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
        Ok(Self { lines })
    }
}
