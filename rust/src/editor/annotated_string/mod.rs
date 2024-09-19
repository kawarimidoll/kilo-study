use crate::prelude::ByteIdx;
use std::cmp::{max, min};
use std::fmt::{Display, Formatter, Result};

use annotated_string_iterator::AnnotatedStringIterator;
use annotated_string_part::AnnotatedStringPart;
mod annotated_string_iterator;
mod annotated_string_part;
use annotation::Annotation;
mod annotation;
pub use annotation_type::AnnotationType;
pub mod annotation_type;

#[derive(Default, Debug)]
pub struct AnnotatedString {
    string: String,
    annotations: Vec<Annotation>,
}

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
    pub fn replace(&mut self, start_byte_idx: ByteIdx, end_byte_idx: ByteIdx, new_string: &str) {
        debug_assert!(start_byte_idx <= end_byte_idx);

        // replace internal string
        let end_byte_idx = min(end_byte_idx, self.string.len());
        if start_byte_idx > end_byte_idx {
            return;
        }
        self.string
            .replace_range(start_byte_idx..end_byte_idx, new_string);

        // update annotations
        let relaced_range_len = end_byte_idx.saturating_sub(start_byte_idx);
        let shortened = new_string.len() < relaced_range_len;
        let len_diff = new_string.len().abs_diff(relaced_range_len);

        if len_diff == 0 {
            // no adjustment is needed in case no change in length
            return;
        }

        self.annotations.iter_mut().for_each(|annotation| {
            annotation.start_byte_idx = if annotation.start_byte_idx >= end_byte_idx {
                // start of the annotation is beyond the insertion point
                // move the start index by the difference in length
                if shortened {
                    annotation.start_byte_idx.saturating_sub(len_diff)
                } else {
                    annotation.start_byte_idx.saturating_add(len_diff)
                }
            } else if annotation.start_byte_idx >= start_byte_idx {
                if shortened {
                    max(
                        start_byte_idx,
                        annotation.start_byte_idx.saturating_sub(len_diff),
                    )
                } else {
                    min(
                        end_byte_idx,
                        annotation.start_byte_idx.saturating_add(len_diff),
                    )
                }
            } else {
                // start byte index is before the insertion point:
                // no adjustment is needed
                annotation.start_byte_idx
            };

            annotation.end_byte_idx = if annotation.end_byte_idx >= end_byte_idx {
                // start of the annotation is beyond the insertion point
                // move the start index by the difference in length
                if shortened {
                    annotation.end_byte_idx.saturating_sub(len_diff)
                } else {
                    annotation.end_byte_idx.saturating_add(len_diff)
                }
            } else if annotation.end_byte_idx >= start_byte_idx {
                if shortened {
                    max(
                        start_byte_idx,
                        annotation.end_byte_idx.saturating_sub(len_diff),
                    )
                } else {
                    min(
                        end_byte_idx,
                        annotation.end_byte_idx.saturating_add(len_diff),
                    )
                }
            } else {
                // start byte index is before the insertion point:
                // no adjustment is needed
                annotation.end_byte_idx
            };

            // filter out empty annotations, in case the previous step resulted in any
        });
        self.annotations.retain(|annotation| {
            annotation.start_byte_idx < annotation.end_byte_idx
                && annotation.start_byte_idx < self.string.len()
        });
    }
}

impl Display for AnnotatedString {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{}", self.string)
    }
}

impl<'a> IntoIterator for &'a AnnotatedString {
    type Item = AnnotatedStringPart<'a>;
    type IntoIter = AnnotatedStringIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AnnotatedStringIterator {
            annotated_string: self,
            current_byte_idx: 0,
        }
    }
}
