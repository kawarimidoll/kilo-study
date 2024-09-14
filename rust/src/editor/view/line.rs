use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
enum GraphemeWidth {
    Half,
    Full,
}

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
}

impl TextFragment {
    pub fn new(grapheme: &str) -> Self {
        let width = match grapheme.width() {
            0 | 1 => GraphemeWidth::Half,
            _ => GraphemeWidth::Full,
        };
        let replacement = Self::get_replacement(grapheme);
        Self {
            grapheme: String::from(grapheme),
            width,
            replacement,
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
}

impl Line {
    pub fn from(string: &str) -> Self {
        Self {
            fragments: Self::string_to_fragments(string),
        }
    }
    fn string_to_fragments(string: &str) -> Vec<TextFragment> {
        string.graphemes(true).map(TextFragment::new).collect()
    }
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
        self.fragments
            .splice(at..at, Self::string_to_fragments(string));
    }
    pub fn remove(&mut self, start: usize, length: usize) {
        let end = start.saturating_add(length);
        self.fragments.splice(start..end, std::iter::empty());
    }
    pub fn append(&mut self, other: &Self) {
        self.fragments.extend(other.fragments.clone());
    }
    pub fn split_off(&mut self, at: usize) -> Self {
        if at > self.len() {
            return Self::default();
        }
        let after = self.fragments.split_off(at);
        Self { fragments: after }
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let result: String = self
            .fragments
            .iter()
            .map(|fragment| fragment.grapheme.clone())
            .collect();
        write!(formatter, "{result}")
    }
}
