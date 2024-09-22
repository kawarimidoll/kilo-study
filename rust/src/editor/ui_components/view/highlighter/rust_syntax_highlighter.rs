use super::SyntaxHighlighter;
use crate::editor::{Annotation, AnnotationType, Line};
use crate::prelude::LineIdx;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct RustSyntaxHighlighter {
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

fn is_number_string(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let mut chars = word.chars();

    // Check if the first character is a digit
    if !chars.next().unwrap().is_ascii_digit() {
        return false;
    }

    let mut seen_dot = false;
    let mut seen_e = false;
    let mut prev_was_digit = true;
    for char in chars {
        match char {
            '0'..='9' => prev_was_digit = true,
            '_' => {
                if !prev_was_digit {
                    // underscores can't be next to each other
                    return false;
                }
                prev_was_digit = false;
            }
            '.' => {
                if seen_dot || seen_e || !prev_was_digit {
                    // can't have more than one dot or an e after a dot
                    return false;
                }
                seen_dot = true;
                prev_was_digit = false;
            }
            'e' | 'E' => {
                if seen_e || !prev_was_digit {
                    // can't have more than one e or an e after a dot
                    return false;
                }
                seen_e = true;
                prev_was_digit = false;
            }
            _ => {
                return false;
            }
        }
    }
    // The last character must be a digit
    prev_was_digit
}

impl SyntaxHighlighter for RustSyntaxHighlighter {
    fn highlight(&mut self, line_idx: LineIdx, line: &Line) {
        let mut result = Vec::new();
        for (start_byte_idx, word) in line.split_word_bound_indices() {
            if is_number_string(word) {
                result.push(Annotation {
                    annotation_type: AnnotationType::Number,
                    start_byte_idx,
                    end_byte_idx: start_byte_idx.saturating_add(word.len()),
                });
            }
        }
        self.highlights.insert(line_idx, result);
    }
    fn get_annotations(&self, line_idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&line_idx)
    }
}
