use crossterm::style::Color;

use super::super::{AnnotationType, HexColor};

pub struct Attribute {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
}

impl From<AnnotationType> for Attribute {
    fn from(annotationtype: AnnotationType) -> Self {
        match annotationtype {
            AnnotationType::Match => Attribute {
                foreground: Some(Color::White),
                background: Some(HexColor::from("#D3D3D3").unwrap().to_color()),
            },
            AnnotationType::SelectedMatch => Attribute {
                foreground: Some(Color::White),
                background: Some(HexColor::from("#FFFF99").unwrap().to_color()),
            },
            AnnotationType::Number => Attribute {
                foreground: Some(HexColor::from("#BC7A21").unwrap().to_color()),
                background: None,
            },
        }
    }
}
