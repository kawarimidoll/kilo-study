use super::AnnotationType;
use crate::prelude::ByteIdx;

#[derive(Debug, Copy, Clone)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub start_byte_idx: ByteIdx,
    pub end_byte_idx: ByteIdx,
}

impl Annotation {
    pub fn shift(&mut self, offset: ByteIdx) {
        self.start_byte_idx = self.start_byte_idx.saturating_add(offset);
        self.end_byte_idx = self.end_byte_idx.saturating_add(offset);
    }
}
