use super::{ColIdx, RowIdx};

#[derive(Copy, Clone, Default)]
pub struct Position {
    // the position of the screen
    pub col: ColIdx,
    pub row: RowIdx,
}
impl Position {
    pub const fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            col: self.col.saturating_sub(other.col),
            row: self.row.saturating_sub(other.row),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_saturating_sub() {
        let result = (Position { col: 10, row: 10 }).saturating_sub(&Position { col: 5, row: 5 });
        assert_eq!(result.col, 5);
        assert_eq!(result.row, 5);

        let result = (Position { col: 3, row: 3 }).saturating_sub(&Position { col: 5, row: 5 });
        assert_eq!(result.col, 0);
        assert_eq!(result.row, 0);
    }
}
