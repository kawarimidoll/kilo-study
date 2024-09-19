use crossterm::style::Color;

#[allow(dead_code)]
pub enum AnnotationType {
    Match,
    SelectedMatch,
}

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
