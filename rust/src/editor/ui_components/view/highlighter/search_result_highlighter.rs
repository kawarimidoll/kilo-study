use super::SyntaxHighlighter;
use crate::editor::{Annotation, AnnotationType, Line, Location};
use crate::prelude::LineIdx;
use std::collections::HashMap;

#[derive(Default)]
pub struct SearchResultHighlighter<'a> {
    matched_word: &'a str,
    selected_match: Option<Location>,
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl<'a> SearchResultHighlighter<'a> {
    pub fn new(matched_word: &'a str, selected_match: Option<Location>) -> Self {
        Self {
            matched_word,
            selected_match,
            highlights: HashMap::new(),
        }
    }
    fn highlight_matched_words(&self, line: &Line, result: &mut Vec<Annotation>) {
        if self.matched_word.is_empty() {
            return;
        }
        line.find_all(self.matched_word, 0..line.string.len())
            .iter()
            .for_each(|(start_byte_idx, _)| {
                result.push(Annotation {
                    annotation_type: AnnotationType::Match,
                    start_byte_idx: *start_byte_idx,
                    end_byte_idx: start_byte_idx.saturating_add(self.matched_word.len()),
                });
            });
    }
    fn highlight_selected_match(&self, line_idx: LineIdx, result: &mut Vec<Annotation>) {
        if let Some(selected_match) = self.selected_match {
            if selected_match.line_idx != line_idx {
                return;
            }
            if self.matched_word.is_empty() {
                return;
            }
            // I'm not sure if this is correct, type inference is hard
            let start_byte_idx = selected_match.grapheme_idx;
            result.push(Annotation {
                annotation_type: AnnotationType::SelectedMatch,
                start_byte_idx,
                end_byte_idx: start_byte_idx.saturating_add(self.matched_word.len()),
            });
        }
    }
}

impl<'a> SyntaxHighlighter for SearchResultHighlighter<'a> {
    fn highlight(&mut self, line_idx: LineIdx, line: &Line) {
        let mut result = Vec::new();
        self.highlight_matched_words(line, &mut result);
        self.highlight_selected_match(line_idx, &mut result);
        self.highlights.insert(line_idx, result);
    }
    fn get_annotations(&self, line_idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&line_idx)
    }
}
