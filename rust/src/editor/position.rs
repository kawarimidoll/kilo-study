pub type Col = usize;
pub type Row = usize;

#[derive(Copy, Clone, Default)]
pub struct Position {
    // the position of the screen
    pub col: Col,
    pub row: Row,
}
impl Position {
    pub const fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            col: self.col.saturating_sub(other.col),
            row: self.row.saturating_sub(other.row),
        }
    }
}
