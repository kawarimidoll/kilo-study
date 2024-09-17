#[derive(Copy, Clone, Default)]
pub struct Location {
    // the position of the document
    pub grapheme_index: usize,
    pub line_index: usize,
}
