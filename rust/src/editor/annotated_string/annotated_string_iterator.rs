use super::{AnnotatedString, AnnotatedStringPart};
use crate::prelude::ByteIdx;
use std::cmp::min;

pub struct AnnotatedStringIterator<'a> {
    pub annotated_string: &'a AnnotatedString,
    pub current_byte_idx: ByteIdx,
}

// Any item the iterator produces lives as long as the iterator itself, because of lifetime 'a
impl<'a> Iterator for AnnotatedStringIterator<'a> {
    type Item = AnnotatedStringPart<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_byte_idx >= self.annotated_string.string.len() {
            return None;
        }

        let start_byte_idx = self.current_byte_idx;
        let mut end_byte_idx = self.annotated_string.string.len();

        // in the active annotation
        // -> find the end of the annotation
        if let Some(annotation) = self
            .annotated_string
            .annotations
            .iter()
            .rfind(|annotation| {
                annotation.start_byte_idx <= self.current_byte_idx
                    && self.current_byte_idx < annotation.end_byte_idx
            })
        {
            let end_byte_idx = min(annotation.end_byte_idx, end_byte_idx);
            self.current_byte_idx = end_byte_idx;
            return Some(AnnotatedStringPart {
                string: &self.annotated_string.string[start_byte_idx..end_byte_idx],
                annotation_type: Some(annotation.annotation_type),
            });
        }

        // in the active annotation
        // -> find the start of the next annotation
        for annotation in &self.annotated_string.annotations {
            if self.current_byte_idx < annotation.start_byte_idx
                && annotation.start_byte_idx < end_byte_idx
            {
                end_byte_idx = annotation.start_byte_idx;
            }
        }
        self.current_byte_idx = end_byte_idx;
        Some(AnnotatedStringPart {
            string: &self.annotated_string.string[start_byte_idx..end_byte_idx],
            annotation_type: None,
        })
    }
}
