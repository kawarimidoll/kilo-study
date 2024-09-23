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
        let mut iterator = line.split_word_bound_indices().peekable();
        while let Some((start_byte_idx, _)) = iterator.next() {
            let remainder = &line[start_byte_idx..];
            if let Some(mut annotation) = annotate_keyword(remainder)
                .or_else(|| annotate_type(remainder))
                .or_else(|| annotate_constant(remainder))
                .or_else(|| annotate_number(remainder))
            {
                annotation.shift(start_byte_idx);
                result.push(annotation);
                while let Some(&(next_byte_idx, _)) = iterator.peek() {
                    if next_byte_idx >= annotation.end_byte_idx {
                        break;
                    }
                    iterator.next();
                }
            }
        }
        self.highlights.insert(line_idx, result);
    }
    fn get_annotations(&self, line_idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&line_idx)
    }
}

fn annotate_next_word<F>(
    string: &str,
    annotation_type: AnnotationType,
    validator: F,
) -> Option<Annotation>
where
    F: Fn(&str) -> bool,
{
    if let Some(word) = string.split_word_bounds().next() {
        if validator(word) {
            return Some(Annotation {
                annotation_type,
                start_byte_idx: 0,
                end_byte_idx: word.len(),
            });
        }
    }
    None
}

fn annotate_number(string: &str) -> Option<Annotation> {
    annotate_next_word(string, AnnotationType::Number, is_number_string)
}
fn annotate_type(string: &str) -> Option<Annotation> {
    annotate_next_word(string, AnnotationType::Type, |word| TYPES.contains(&word))
}
fn annotate_keyword(string: &str) -> Option<Annotation> {
    annotate_next_word(string, AnnotationType::Keyword, |word| {
        KEYWORDS.contains(&word)
    })
}
fn annotate_constant(string: &str) -> Option<Annotation> {
    annotate_next_word(string, AnnotationType::Constant, |word| {
        CONSTANTS.contains(&word)
    })
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
