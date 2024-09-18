use std::fmt::{Display, Formatter, Result};
use std::ops::{Deref, Range};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
enum GraphemeWidth {
    Half,
    Full,
}

type GraphemeIdx = usize;
type ByteIdx = usize;

impl GraphemeWidth {
    fn saturating_add(&self, other: usize) -> usize {
        other.saturating_add(self.as_usize())
    }
    fn as_usize(&self) -> usize {
        match self {
            Self::Half => 1,
            Self::Full => 2,
        }
    }
}

#[derive(Clone)]
struct TextFragment {
    grapheme: String,
    width: GraphemeWidth,
    replacement: Option<String>,
    start_byte_idx: ByteIdx,
}

impl TextFragment {
    pub fn new(start_byte_idx: ByteIdx, grapheme: &str) -> Self {
        let replacement = Self::get_replacement(grapheme);
        // for now, replacement character is always Half width
        let width = if grapheme.width() <= 1 || replacement.is_some() {
            GraphemeWidth::Half
        } else {
            GraphemeWidth::Full
        };
        Self {
            grapheme: String::from(grapheme),
            width,
            replacement,
            start_byte_idx,
        }
    }
    fn get_replacement(grapheme: &str) -> Option<String> {
        let g_width = grapheme.width();
        match grapheme {
            " " => None,
            "\t" => Some("→".to_string()),
            _ if g_width > 0 && grapheme.trim().is_empty() => Some("␣".to_string()),
            _ if g_width == 0 => Some("·".to_string()),
            _ => {
                let mut chars = grapheme.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        // let replacement = ((ch as u8) + 64) as char;
                        // return Some(format!("^{replacement}").to_string());
                        return Some("▯".to_string());
                    }
                }
                None
            }
        }
    }
}

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
}

impl Line {
    pub fn from(string: &str) -> Self {
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
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert_str(fragment.start_byte_idx, string);
        } else {
            self.string.push_str(string);
        }
        self.rebuild_fragments();
    }
    pub fn remove(&mut self, start: GraphemeIdx, length: GraphemeIdx) {
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
        self.fragments
            .iter()
            .position(|fragment| fragment.start_byte_idx >= byte_idx)
            .map_or(0, |grapheme_idx| grapheme_idx)
    }
    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        self.fragments
            .get(grapheme_idx)
            .map_or(0, |fragment| fragment.start_byte_idx)
    }
    pub fn search_forward(&self, query: &str, from_grapheme_idx: GraphemeIdx) -> Option<GraphemeIdx> {
        if from_grapheme_idx >= self.grapheme_count() {
            return None;
        }
        let start_byte_idx = self.grapheme_idx_to_byte_idx(from_grapheme_idx);
        self.string
            .get(start_byte_idx..)
            .and_then(|substr| substr.find(query))
            .map(|byte_idx| self.byte_idx_to_grapheme_idx(byte_idx.saturating_add(start_byte_idx)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grapheme_width() {
        let half = GraphemeWidth::Half;
        let full = GraphemeWidth::Full;
        assert_eq!(half.saturating_add(3), 4);
        assert_eq!(full.saturating_add(3), 5);
    }

    #[test]
    fn test_text_fragment() {
        // normal character
        let f = TextFragment::new(0, "a");
        assert_eq!(f.grapheme, "a");
        assert_eq!(matches!(f.width, GraphemeWidth::Half), true);
        assert_eq!(f.replacement, None);

        // full-width character
        let f = TextFragment::new(0, "緑");
        assert_eq!(f.grapheme, "緑");
        assert_eq!(matches!(f.width, GraphemeWidth::Full), true);
        assert_eq!(f.replacement, None);

        // zero-width character
        let f = TextFragment::new(0, " ");
        assert_eq!(f.grapheme, " ");
        assert_eq!(matches!(f.width, GraphemeWidth::Half), true);
        assert_eq!(f.replacement, Some(String::from("␣")));

        // zero-width character
        let f = TextFragment::new(0, "​");
        assert_eq!(f.grapheme, "​");
        assert_eq!(matches!(f.width, GraphemeWidth::Half), true);
        assert_eq!(f.replacement, Some(String::from("·")));

        // tab
        let f = TextFragment::new(0, "\t");
        assert_eq!(f.grapheme, "\t");
        assert_eq!(matches!(f.width, GraphemeWidth::Half), true);
        assert_eq!(f.replacement, Some(String::from("→")));

        // control character
        let f = TextFragment::new(0, "");
        assert_eq!(f.grapheme, "");
        assert_eq!(matches!(f.width, GraphemeWidth::Half), true);
        assert_eq!(f.replacement, Some(String::from("▯")));
        // assert_eq!(f.replacement, Some(String::from("^G")));
    }
}
