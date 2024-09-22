use super::AnnotationType;
use crate::prelude::ByteIdx;

#[derive(Debug,Copy,Clone)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub start_byte_idx: ByteIdx,
    pub end_byte_idx: ByteIdx,
}
