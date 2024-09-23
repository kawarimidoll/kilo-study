#[derive(Clone, Copy, Debug)]
pub enum AnnotationType {
    Match,
    SelectedMatch,
    Number,
    Keyword,
    Constant,
    Type,
    Char,
    LifetimeSpecifier,
}
