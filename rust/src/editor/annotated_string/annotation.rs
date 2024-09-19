use super::AnnotationType;
use crate::editor::ByteIdx;

#[derive(Debug)]
pub struct Annotation {
    pub annotation_type: AnnotationType,
    pub start_byte_idx: ByteIdx,
    pub end_byte_idx: ByteIdx,
}
