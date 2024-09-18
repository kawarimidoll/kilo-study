use super::{ByteIdx, GraphemeWidth};
use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
pub struct TextFragment {
    pub grapheme: String,
    pub width: GraphemeWidth,
    pub replacement: Option<String>,
    pub start_byte_idx: ByteIdx,
}

impl TextFragment {
    pub fn new(start_byte_idx: ByteIdx, grapheme: &str) -> Self {
        let replacement = Self::get_replacement(grapheme);
        // for now, replacement character is always Half width
        // TODO: handle full-width replacement characters
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
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
