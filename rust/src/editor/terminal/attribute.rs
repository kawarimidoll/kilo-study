use crossterm::style::Color;

use super::super::AnnotationType;

pub struct Attribute {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}

impl From<AnnotationType> for Attribute {
    fn from(annotationtype: AnnotationType) -> Self {
        match annotationtype {
            AnnotationType::Match => Attribute {
                foreground: Some(Color::White),
                background: Some(Color::Blue),
            },
            AnnotationType::SelectedMatch => Attribute {
                foreground: Some(Color::White),
                background: Some(Color::Cyan),
            },
        }
    }
}
