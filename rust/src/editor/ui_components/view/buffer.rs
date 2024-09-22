use super::{GraphemeIdx, Highlighter, Line};
use crate::editor::annotated_string::AnnotatedString;
use crate::prelude::{LineIdx, Location};
use std::fs::{read_to_string, File};
use std::io::{Error, Write};
use std::ops::Range;

use crate::editor::file_info::FileInfo;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_info: FileInfo,
    pub dirty: usize,
}

impl Buffer {
    pub fn height(&self) -> LineIdx {
        self.lines.len()
    }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    pub fn grapheme_count(&self, line_idx: LineIdx) -> GraphemeIdx {
        self.lines.get(line_idx).map_or(0, Line::grapheme_count)
    }
    pub fn width_until(&self, line_idx: LineIdx, until: GraphemeIdx) -> GraphemeIdx {
        self.lines
            .get(line_idx)
            .map_or(0, |line| line.width_until(until))
    }

    pub fn get_highlighted_substring(
        &self,
        line_idx: LineIdx,
        range: Range<GraphemeIdx>,
        highlighter: &Highlighter,
    ) -> Option<AnnotatedString> {
        self.lines.get(line_idx).map(|line| {
            line.get_annotated_visible_substr(range, Some(&highlighter.get_annotations(line_idx)))
        })
    }
    pub fn highlight(&self, line_idx: LineIdx, highlighter: &mut Highlighter) {
        if let Some(line) = self.lines.get(line_idx) {
            highlighter.highlight(line_idx, line);
        }
    }
    pub fn insert_newline(&mut self, at: Location) -> bool {
        let Location {
            grapheme_idx,
            line_idx,
        } = at;
        if line_idx >= self.height() {
            self.lines.push(Line::default());
        } else {
            // we have a valid line_idx
            let second_half = self.lines[line_idx].split_off(grapheme_idx);
            self.lines.insert(line_idx.saturating_add(1), second_half);
        }
        self.dirty = self.dirty.saturating_add(1);
        true
    }
    pub fn remove_char(&mut self, at: Location) -> bool {
        let Location {
            grapheme_idx,
            line_idx,
        } = at;
        // out of bounds
        if line_idx >= self.height() {
            return false;
        }

        // below here, we have a valid line_idx
        if grapheme_idx < self.lines[line_idx].len() {
            self.lines[line_idx].remove(grapheme_idx, 1);
        } else if line_idx < self.height().saturating_sub(1) {
            let next_line = self.lines.remove(line_idx.saturating_add(1));
            self.lines[line_idx].append(&next_line);
        } else {
            // the last line, the last character
            return false;
        }
        self.dirty = self.dirty.saturating_add(1);
        true
    }
    pub fn insert_char(&mut self, c: char, at: Location) -> bool {
        debug_assert!(at.line_idx <= self.height());
        let Location {
            grapheme_idx,
            line_idx,
        } = at;
        // out of bounds
        if line_idx > self.height() {
            return false;
        }

        let string = c.to_string();

        // append a new line
        if line_idx == self.height() {
            self.lines.push(Line::from(&string));
            self.dirty = self.dirty.saturating_add(1);
            return true;
        }

        // insert a new character in an existing line
        if let Some(line) = self.lines.get_mut(line_idx) {
            line.insert(grapheme_idx, &string);
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
    pub fn search_forward(&self, query: &str, from: Location) -> Option<Location> {
        // search from the current line to the end
        for (line_idx, line) in self.lines.iter().enumerate().skip(from.line_idx) {
            let from_grapheme_idx = if line_idx == from.line_idx {
                from.grapheme_idx
            } else {
                0
            };
            if let Some(grapheme_idx) = line.search_forward(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }
        // wrap around to the beginning
        for (line_idx, line) in self.lines.iter().enumerate().take(from.line_idx) {
            let from_grapheme_idx = 0;
            if let Some(grapheme_idx) = line.search_forward(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }
        None
    }
    pub fn search_backward(&self, query: &str, from: Location) -> Option<Location> {
        if query.is_empty() {
            return None;
        }
        let mut is_first = true;
        for (line_idx, line) in self
            .lines
            .iter()
            .enumerate()
            .rev()
            .cycle()
            .skip(
                self.lines
                    .len()
                    .saturating_sub(from.line_idx)
                    .saturating_sub(1),
            )
            .take(self.lines.len().saturating_add(1))
        {
            let from_grapheme_idx = if is_first {
                is_first = false;
                from.grapheme_idx
            } else {
                line.grapheme_count()
            };
            if let Some(grapheme_idx) = line.search_backward(query, from_grapheme_idx) {
                return Some(Location {
                    grapheme_idx,
                    line_idx,
                });
            }
        }
        None
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
            Ok(())
        } else {
            Err(Error::new(std::io::ErrorKind::Other, "No file path"))
        }
    }
}
