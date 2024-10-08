use super::is_number_string;
use super::SyntaxHighlighter;
use crate::editor::{Annotation, AnnotationType, Line};
use crate::prelude::LineIdx;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct RustSyntaxHighlighter {
    highlights: Vec<Vec<Annotation>>,
    ml_comment_balance: usize,
    in_string: bool,
}

impl RustSyntaxHighlighter {
    fn annotate_ml_comment(&mut self, string: &str) -> Option<Annotation> {
        let mut chars = string.char_indices().peekable();
        while let Some((_, c)) = chars.next() {
            // handle potential comment start
            if c == '/' {
                if let Some((_, '*')) = chars.peek() {
                    self.ml_comment_balance = self.ml_comment_balance.saturating_add(1);
                    chars.next();
                }
            } else if self.ml_comment_balance == 0 {
                return None;
            } else if c == '*' {
                if let Some((idx, '/')) = chars.peek() {
                    self.ml_comment_balance = self.ml_comment_balance.saturating_sub(1);
                    if self.ml_comment_balance == 0 {
                        return Some(Annotation {
                            annotation_type: AnnotationType::Comment,
                            start_byte_idx: 0,
                            end_byte_idx: idx.saturating_add(1),
                        });
                    }
                    chars.next();
                }
            }
        }
        // if still not exit at this point,
        // we might be in a multi-line comment
        // then annotate the entire lines as a comment
        (self.ml_comment_balance > 0).then_some(Annotation {
            annotation_type: AnnotationType::Comment,
            start_byte_idx: 0,
            end_byte_idx: string.len(),
        })
    }
    fn annotate_string(&mut self, string: &str) -> Option<Annotation> {
        let mut chars = string.char_indices();
        while let Some((idx, c)) = chars.next() {
            if c == '\\' {
                // skip escaped character
                chars.next();
                continue;
            }
            // handle potential string start / end
            if c == '"' {
                if self.in_string {
                    // end of string
                    self.in_string = false;
                    return Some(Annotation {
                        annotation_type: AnnotationType::String,
                        start_byte_idx: 0,
                        end_byte_idx: idx.saturating_add(1),
                    });
                }
                // start of string
                self.in_string = true;
            }
            if !self.in_string {
                return None;
            }
        }
        // if still not exit at this point,
        // we might be in a multi-line string
        // then annotate the entire lines as a string
        self.in_string.then_some(Annotation {
            annotation_type: AnnotationType::String,
            start_byte_idx: 0,
            end_byte_idx: string.len(),
        })
    }
    fn initial_annotation(&mut self, line: &Line) -> Option<Annotation> {
        // select the first annotation in the line
        if self.in_string {
            self.annotate_string(line)
        } else if self.ml_comment_balance > 0 {
            self.annotate_ml_comment(line)
        } else {
            None
        }
    }
}
impl SyntaxHighlighter for RustSyntaxHighlighter {
    fn highlight(&mut self, line_idx: LineIdx, line: &Line) {
        debug_assert_eq!(line_idx, self.highlights.len());
        let mut result = Vec::new();
        let mut iterator = line.split_word_bound_indices().peekable();
        // handle dangling multi-line annotations
        if let Some(annotation) = self.initial_annotation(line) {
            // no need to shift here because idx is 0
            result.push(annotation);
            // skip over any subsequent word which has already been annotated
            while let Some(&(next_byte_idx, _)) = iterator.peek() {
                if next_byte_idx >= annotation.end_byte_idx {
                    break;
                }
                iterator.next();
            }
        }
        while let Some((start_byte_idx, _)) = iterator.next() {
            let remainder = &line[start_byte_idx..];
            if let Some(mut annotation) = self
                .annotate_ml_comment(remainder)
                .or_else(|| self.annotate_string(remainder))
                .or_else(|| annotate_single_line_comment(remainder))
                .or_else(|| annotate_char(remainder))
                .or_else(|| annotate_lifetime_specifier(remainder))
                .or_else(|| annotate_keyword(remainder))
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
        self.highlights.push(result);
    }
    fn get_annotations(&self, line_idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(line_idx)
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
fn annotate_char(string: &str) -> Option<Annotation> {
    // like: 'a', '\n', '\'', '\\', 'single_word'
    let mut iter = string.split_word_bound_indices().peekable();
    // handle opening quote
    if let Some((_, "\'")) = iter.next() {
        if let Some((_, "\\")) = iter.peek() {
            // skip escape character
            iter.next();
        }
        // skip character
        iter.next();
        if let Some((idx, "\'")) = iter.next() {
            // include closing quote
            let end_byte_idx = idx.saturating_add(1);
            return Some(Annotation {
                annotation_type: AnnotationType::Char,
                start_byte_idx: 0,
                end_byte_idx,
            });
        }
    }
    None
}
fn annotate_lifetime_specifier(string: &str) -> Option<Annotation> {
    // like: 'a, '_, '123
    let mut iter = string.split_word_bound_indices();
    // handle start quote
    if let Some((_, "\'")) = iter.next() {
        if let Some((idx, next_word)) = iter.next() {
            return Some(Annotation {
                annotation_type: AnnotationType::LifetimeSpecifier,
                start_byte_idx: 0,
                end_byte_idx: idx.saturating_add(next_word.len()),
            });
        }
    }
    None
}
fn annotate_single_line_comment(string: &str) -> Option<Annotation> {
    if string.starts_with("//") {
        return Some(Annotation {
            annotation_type: AnnotationType::Comment,
            start_byte_idx: 0,
            end_byte_idx: string.len(),
        });
    }
    None
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
