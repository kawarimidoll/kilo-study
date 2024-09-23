use super::is_number_string;
use super::SyntaxHighlighter;
use crate::editor::{Annotation, AnnotationType, Line};
use crate::prelude::LineIdx;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct RustSyntaxHighlighter {
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl SyntaxHighlighter for RustSyntaxHighlighter {
    fn highlight(&mut self, line_idx: LineIdx, line: &Line) {
        let mut result = Vec::new();
        for (start_byte_idx, word) in line.split_word_bound_indices() {
            let end_byte_idx = start_byte_idx.saturating_add(word.len());
            if KEYWORDS.contains(&word) {
                result.push(Annotation {
                    annotation_type: AnnotationType::Keyword,
                    start_byte_idx,
                    end_byte_idx,
                });
            } else if TYPES.contains(&word) {
                result.push(Annotation {
                    annotation_type: AnnotationType::Type,
                    start_byte_idx,
                    end_byte_idx,
                });
            } else if CONSTANTS.contains(&word) {
                result.push(Annotation {
                    annotation_type: AnnotationType::Constant,
                    start_byte_idx,
                    end_byte_idx,
                });
            } else if is_number_string(word) {
                result.push(Annotation {
                    annotation_type: AnnotationType::Number,
                    start_byte_idx,
                    end_byte_idx,
                });
            }
        }
        self.highlights.insert(line_idx, result);
    }
    fn get_annotations(&self, line_idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&line_idx)
    }
}
const TYPES: [&str; 22] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "Option", "Result", "String", "str", "Vec", "HashMap",
];
const CONSTANTS: [&str; 6] = ["true", "false", "None", "Some", "Ok", "Err"];
const KEYWORDS: [&str; 51] = [
    "Self",
    "abstract",
    "as",
    "async",
    "await",
    "become",
    "box",
    "break",
    "const",
    "continue",
    "crate",
    "do",
    "dyn",
    "else",
    "enum",
    "extern",
    "final",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "macro",
    "macro_rules",
    "match",
    "mod",
    "move",
    "mut",
    "override",
    "priv",
    "pub",
    "ref",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "try",
    "type",
    "typeof",
    "union",
    "unsafe",
    "unsized",
    "use",
    "virtual",
    "where",
    "while",
    "yield",
];
