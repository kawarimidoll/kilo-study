use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
}

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
        let replacement = match grapheme.width() {
            0 => Some('·'),
            _ => None,
        };
        Self {
            grapheme: String::from(grapheme),
            width,
            replacement,
        }
    }
}

pub struct Line {
    fragments: Vec<TextFragment>,
}

impl Line {
    pub fn from(string: &str) -> Self {
        Self {
            fragments: string.graphemes(true).map(TextFragment::new).collect(),
        }
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
}
