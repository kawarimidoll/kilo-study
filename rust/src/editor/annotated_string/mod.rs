use crate::editor::ByteIdx;
use std::fmt::{Display, Formatter, Result};

use annotated_string_iterator::AnnotatedStringIterator;
mod annotated_string_iterator;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum AnnotationType {
    Match,
    SelectedMatch,
}

#[allow(dead_code)]
pub struct Annotation {
    annotation_type: AnnotationType,
    start_byte_idx: ByteIdx,
    end_byte_idx: ByteIdx,
}

#[allow(dead_code)]
pub struct AnnotatedStringPart<'a> {
    string: &'a str,
    annotation_type: Option<AnnotationType>,
}

#[allow(dead_code)]
pub struct AnnotatedString {
    string: String,
    annotations: Vec<Annotation>,
}

#[allow(dead_code)]
impl AnnotatedString {
    pub fn from(string: &str) -> Self {
        Self {
            string: String::from(string),
            annotations: Vec::new(),
        }
    }
    pub fn push(
        &mut self,
        annotation_type: AnnotationType,
        start_byte_idx: ByteIdx,
        end_byte_idx: ByteIdx,
    ) {
        debug_assert!(start_byte_idx <= end_byte_idx);
        self.annotations.push(Annotation {
            annotation_type,
            start_byte_idx,
            end_byte_idx,
        });
    }
}

impl Display for AnnotatedString {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{}", self.string)
    }
}

impl <'a> IntoIterator for &'a AnnotatedString {
    type Item = AnnotatedStringPart<'a>;
    type IntoIter = AnnotatedStringIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AnnotatedStringIterator {
            annotated_string: self,
            current_byte_idx: 0,
        }
    }
}
