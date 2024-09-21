use crate::editor::{Annotation, AnnotationType, Line};
use crate::prelude::{LineIdx, Location};
use std::collections::HashMap;

#[derive(Default)]
pub struct Highlighter<'a> {
    matched_word: Option<&'a str>,
    selected_match: Option<Location>,
    highlights: HashMap<LineIdx, Vec<Annotation>>,
}

impl<'a> Highlighter<'a> {
    pub fn new(matched_word: Option<&'a str>, selected_match: Option<Location>) -> Self {
        Self {
            matched_word,
            selected_match,
            highlights: HashMap::new(),
        }
    }
    pub fn get_annotations(&self, line_idx: LineIdx) -> Option<&Vec<Annotation>> {
        self.highlights.get(&line_idx)
    }
    fn highlight_digits(line: &Line, result: &mut Vec<Annotation>) {
        line.chars().enumerate().for_each(|(idx, ch)| {
            if ch.is_ascii_digit() {
                result.push(Annotation {
                    annotation_type: AnnotationType::Digit,
                    start_byte_idx: idx,
                    end_byte_idx: idx.saturating_add(1),
                });
            }
        });
    }
    fn highlight_matched_words(&self, line: &Line, result: &mut Vec<Annotation>) {
        if let Some(matched_word) = self.matched_word {
            if matched_word.is_empty() {
                return;
            }
            line.find_all(matched_word, 0..line.string.len())
                .iter()
                .for_each(|(start_byte_idx, _)| {
                    result.push(Annotation {
                        annotation_type: AnnotationType::Match,
                        start_byte_idx: *start_byte_idx,
                        end_byte_idx: start_byte_idx.saturating_add(matched_word.len()),
                    });
                });
        }
    }
    fn highlight_selected_match(&self, line_idx: LineIdx, result: &mut Vec<Annotation>) {
        if let Some(selected_match) = self.selected_match {
            if selected_match.line_idx != line_idx {
                return;
            }
            if let Some(matched_word) = self.matched_word {
                if matched_word.is_empty() {
                    return;
                }
                // I'm not sure if this is correct, type inference is hard
                let start_byte_idx = selected_match.grapheme_idx;
                result.push(Annotation {
                    annotation_type: AnnotationType::SelectedMatch,
                    start_byte_idx,
                    end_byte_idx: start_byte_idx.saturating_add(matched_word.len()),
                });
            }
        }
    }
    pub fn highlight(&mut self, line_idx: LineIdx, line: &Line) {
        let mut result = Vec::new();
        Self::highlight_digits(line, &mut result);
        self.highlight_matched_words(line, &mut result);
        self.highlight_selected_match(line_idx, &mut result);
        self.highlights.insert(line_idx, result);
    }
}
