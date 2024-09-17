#[derive(Copy, Clone, Default)]
pub struct Location {
    // the position of the document
    pub grapheme_idx: usize,
    pub line_idx: usize,
}
