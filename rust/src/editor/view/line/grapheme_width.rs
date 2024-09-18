#[derive(Copy, Clone)]
pub enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    pub fn saturating_add(self, other: usize) -> usize {
        other.saturating_add(self.as_usize())
    }
    pub fn as_usize(self) -> usize {
        match self {
            Self::Half => 1,
            Self::Full => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturating_add() {
        let half = GraphemeWidth::Half;
        let full = GraphemeWidth::Full;
        assert_eq!(half.saturating_add(3), 4);
        assert_eq!(full.saturating_add(3), 5);
    }
}
