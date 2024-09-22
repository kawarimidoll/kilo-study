use super::SyntaxHighlighter;
use crate::editor::{Annotation, AnnotationType, Line};
use crate::prelude::LineIdx;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
use super::is_number_string;

#[derive(Default)]
pub struct RustSyntaxHighlighter {
    highlights: HashMap<LineIdx, Vec<Annotation>>,
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
