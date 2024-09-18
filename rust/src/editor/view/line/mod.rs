use grapheme_width::GraphemeWidth;
use std::fmt::{Display, Formatter, Result};
use std::ops::{Deref, Range};
use unicode_segmentation::UnicodeSegmentation;
mod grapheme_width;
mod text_fragment;
use text_fragment::TextFragment;

pub type GraphemeIdx = usize;
pub type ByteIdx = usize;

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
}

impl Line {
    pub fn from(string: &str) -> Self {
        debug_assert!(string.is_empty() || string.lines().count() == 1);
        Self {
            fragments: Self::string_to_fragments(string),
            string: String::from(string),
        }
    }
    fn rebuild_fragments(&mut self) {
        self.fragments = Self::string_to_fragments(&self.string);
    }
    fn string_to_fragments(string: &str) -> Vec<TextFragment> {
        string
            .grapheme_indices(true)
            .map(|(start_byte_idx, grapheme)| TextFragment::new(start_byte_idx, grapheme))
            .collect()
    }
    // fn fragments_to_string(fragments: Vec<TextFragment>) -> String {
    //     fragments
    //         .iter()
    //         .map(|fragment| fragment.grapheme.clone())
    //         .collect()
    // }
    pub fn get_visible_graphemes(&self, range: Range<GraphemeIdx>) -> String {
        let mut result = String::new();
        let start = range.start;
        let end = range.end;

        if start >= end {
            return result;
        }
        let mut current_pos = 0;

        for fragment in &self.fragments {
            if current_pos >= end {
                break;
            }
            let fragment_end = fragment.width.saturating_add(current_pos);
            if fragment_end >= start {
                if fragment_end >= end || current_pos < start {
                    // boundary of screen
                    result.push('…');
                } else if let Some(char) = fragment.replacement.clone() {
                    // use replacement character for empty graphemes
                    result.push_str(&char);
                } else {
                    // use the original grapheme
                    result.push_str(&fragment.grapheme);
                }
            }
            current_pos = fragment_end;
        }

        result
    }
    pub fn grapheme_count(&self) -> GraphemeIdx {
        self.fragments.len()
    }
    pub fn width_until(&self, grapheme_idx: GraphemeIdx) -> GraphemeIdx {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(|fragment| fragment.width.as_usize())
            .sum()
    }

    pub fn insert(&mut self, at: GraphemeIdx, string: &str) {
        debug_assert!(at.saturating_sub(1) <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert_str(fragment.start_byte_idx, string);
        } else {
            self.string.push_str(string);
        }
        self.rebuild_fragments();
    }
    pub fn remove(&mut self, start: GraphemeIdx, length: GraphemeIdx) {
        debug_assert!(start <= self.grapheme_count());
        if let Some(start_fragment) = self.fragments.get(start) {
            let end = start.saturating_add(length);
            let start_byte_idx = start_fragment.start_byte_idx;
            if let Some(end_fragment) = self.fragments.get(end) {
                let end_byte_idx = end_fragment.start_byte_idx;
                self.string.drain(start_byte_idx..end_byte_idx);
            } else {
                self.string.drain(start_byte_idx..);
            }
            self.rebuild_fragments();
        }
    }
    pub fn append(&mut self, other: &Self) {
        self.insert(self.len(), &other.string);
    }

    pub fn split_off(&mut self, at: GraphemeIdx) -> Self {
        if let Some(fragment) = self.fragments.get(at) {
            let reminder = self.string.split_off(fragment.start_byte_idx);
            self.rebuild_fragments();
            Self::from(&reminder)
        } else {
            Self::default()
        }
    }
    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> GraphemeIdx {
        debug_assert!(byte_idx <= self.string.len());
        self.fragments
            .iter()
            .position(|fragment| fragment.start_byte_idx >= byte_idx)
            .map_or_else(
                || {
                    #[cfg(debug_assertions)]
                    panic!("byte_idx_to_grapheme_idx: byte_idx: {byte_idx:?} not found");
                    #[cfg(not(debug_assertions))]
                    0
                },
                |grapheme_idx| grapheme_idx,
            )
    }
    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        debug_assert!(grapheme_idx <= self.grapheme_count());
        self.fragments.get(grapheme_idx).map_or_else(
            || {
                #[cfg(debug_assertions)]
                panic!("grapheme_idx_to_byte_idx: grapheme_idx: {grapheme_idx:?} not found");
                #[cfg(not(debug_assertions))]
                0
            },
            |fragment| fragment.start_byte_idx,
        )
    }
    pub fn search_forward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if from_grapheme_idx >= self.grapheme_count() {
            return None;
        }
        let start_byte_idx = self.grapheme_idx_to_byte_idx(from_grapheme_idx);
        self.string
            .get(start_byte_idx..)
            .and_then(|substr| substr.find(query))
            .map(|byte_idx| self.byte_idx_to_grapheme_idx(byte_idx.saturating_add(start_byte_idx)))
    }
    pub fn search_backward(
        &self,
        query: &str,
        from_grapheme_idx: GraphemeIdx,
    ) -> Option<GraphemeIdx> {
        debug_assert!(from_grapheme_idx <= self.grapheme_count());
        if from_grapheme_idx == 0 {
            return None;
        }
        let end_byte_idx = if from_grapheme_idx == self.grapheme_count() {
            self.string.len()
        } else {
            self.grapheme_idx_to_byte_idx(from_grapheme_idx)
        };
        self.string
            .get(..end_byte_idx)
            .and_then(|substr| substr.match_indices(query).last())
            .map(|(byte_idx, _)| self.byte_idx_to_grapheme_idx(byte_idx))
    }
    #[allow(dead_code)]
    pub fn search_ignore_case(&self, query: &str) -> Option<GraphemeIdx> {
        self.string
            .to_lowercase()
            .find(&query.to_lowercase())
            .map(|byte_idx| self.byte_idx_to_grapheme_idx(byte_idx))
    }
}

impl Display for Line {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{}", self.string)
    }
}
impl Deref for Line {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.string
    }
}
