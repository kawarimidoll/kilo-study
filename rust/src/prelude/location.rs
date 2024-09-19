use crate::prelude::{GraphemeIdx, LineIdx};

#[derive(Copy, Clone, Default)]
pub struct Location {
    // the position of the document
    pub grapheme_idx: GraphemeIdx,
    pub line_idx: LineIdx,
}
