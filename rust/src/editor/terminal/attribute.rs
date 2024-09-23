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
            AnnotationType::Keyword => Attribute {
                foreground: Some(HexColor::from("#2393D3").unwrap().to_color()),
                background: None,
            },
            AnnotationType::Constant => Attribute {
                foreground: Some(HexColor::from("#23D323").unwrap().to_color()),
                background: None,
            },
            AnnotationType::Type => Attribute {
                foreground: Some(HexColor::from("#9323D3").unwrap().to_color()),
                background: None,
            },
            AnnotationType::Char => Attribute {
                foreground: Some(HexColor::from("#D2B900").unwrap().to_color()),
                background: None,
            },
            AnnotationType::LifetimeSpecifier=> Attribute {
                foreground: Some(HexColor::from("#66CDE2").unwrap().to_color()),
                background: None,
            },
            AnnotationType::Comment => Attribute {
                foreground: Some(HexColor::from("#929292").unwrap().to_color()),
                background: None,
            },
            AnnotationType::String => Attribute {
                foreground: Some(HexColor::from("#FFB366").unwrap().to_color()),
                background: None,
            },
        }
    }
}
