use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
enum GraphemeWidth {
    Half,
    Full,
}

// TODO: control characters are not displayed properly

impl GraphemeWidth {
    fn saturating_add(&self, other: usize) -> usize {
        match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
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
    replacement: Option<char>,
    start_byte_index: usize,
}

impl TextFragment {
    pub fn new(start_byte_index: usize, grapheme: &str) -> Self {
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
            start_byte_index,
        }
    }
    fn get_replacement(grapheme: &str) -> Option<char> {
        let g_width = grapheme.width();
        match grapheme {
            " " => None,
            "\t" => Some('→'),
            _ if g_width > 0 && grapheme.trim().is_empty() => Some('␣'),
            _ if g_width == 0 => {
                // it doesn't seem to work properly...
                let mut chars = grapheme.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _ => None,
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
            .map(|(start_byte_index, grapheme)| TextFragment::new(start_byte_index, grapheme))
            .collect()
    }
    // fn fragments_to_string(fragments: Vec<TextFragment>) -> String {
    //     fragments
    //         .iter()
    //         .map(|fragment| fragment.grapheme.clone())
    //         .collect()
    // }
    pub fn get(&self, range: Range<usize>) -> String {
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
                } else if let Some(char) = fragment.replacement {
                    // use replacement character for empty graphemes
                    result.push(char);
                } else {
                    // use the original grapheme
                    result.push_str(&fragment.grapheme);
                }
            }
            current_pos = fragment_end;
        }

        result
    }
    pub fn len(&self) -> usize {
        self.fragments.len()
    }
    pub fn width_until(&self, end: usize) -> usize {
        self.fragments
            .iter()
            .take(end)
            .map(|fragment| fragment.width.as_usize())
            .sum()
    }

    pub fn insert(&mut self, at: usize, string: &str) {
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert_str(fragment.start_byte_index, string);
        } else {
            self.string.push_str(string);
        }
        self.rebuild_fragments();
    }
    pub fn remove(&mut self, start: usize, length: usize) {
        if let Some(start_fragment) = self.fragments.get(start) {
            let end = start.saturating_add(length);
            let start_byte_index = start_fragment.start_byte_index;
            if let Some(end_fragment) = self.fragments.get(end) {
                let end_byte_index = end_fragment.start_byte_index;
                self.string.drain(start_byte_index..end_byte_index);
            } else {
                self.string.drain(start_byte_index..);
            }
            self.rebuild_fragments();
        }
    }
    pub fn append(&mut self, other: &Self) {
        self.insert(self.len(), &other.string);
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        if let Some(fragment) = self.fragments.get(at) {
            let reminder = self.string.split_off(fragment.start_byte_index);
            self.rebuild_fragments();
            Self::from(&reminder)
        } else {
            Self::default()
        }
    }
    fn byte_idx_to_grapheme_idx(&self, byte_idx: usize) -> usize {
        for (grapheme_idx, fragment) in self.fragments.iter().enumerate() {
            if fragment.start_byte_index >= byte_idx {
                return grapheme_idx;
            }
        }

        #[cfg(debug_assertions)]
        panic!("byte_idx_to_grapheme_idx: Invalid byte_index: {byte_idx:?}");

        #[cfg(not(debug_assertions))]
        0
    }
    pub fn search(&self, query:&str) -> Option<usize> {
        self.string.find(query)
            .map(|byte_idx| self.byte_idx_to_grapheme_idx(byte_idx))
    }
    pub fn search_ignore_case(&self, query:&str) -> Option<usize> {
        self.string.to_lowercase().find(&query.to_lowercase())
            .map(|byte_idx| self.byte_idx_to_grapheme_idx(byte_idx))
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.string)
    }
}
