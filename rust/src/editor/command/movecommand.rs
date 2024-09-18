use crossterm::event::{
    KeyCode::{
        Char, Down, End, Home, Left, PageDown, PageUp, Right, Up,
    },
    KeyEvent, KeyModifiers,
};
use std::convert::TryFrom;

#[derive(Copy, Clone)]
pub enum Move {
    Down,
    EndOfLine,
    StartOfLine,
    Left,
    PageDown,
    PageUp,
    Right,
    Up,
}
impl TryFrom<KeyEvent> for Move {
    type Error = String;
    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        match (code, modifiers) {
            (Down, _) | (Char('n'), KeyModifiers::CONTROL) => Ok(Self::Down),
            (End, _) | (Char('e'), KeyModifiers::CONTROL) => Ok(Self::EndOfLine),
            (Home, _) | (Char('a'), KeyModifiers::CONTROL) => Ok(Self::StartOfLine),
            (Left, _) | (Char('b'), KeyModifiers::CONTROL) => Ok(Self::Left),
            (PageDown, _) => Ok(Self::PageDown),
            (PageUp, _) => Ok(Self::PageUp),
            (Right, _) | (Char('f'), KeyModifiers::CONTROL) => Ok(Self::Right),
            (Up, _) | (Char('p'), KeyModifiers::CONTROL) => Ok(Self::Up),
            _ => Err(format!(
                "Unrecognized key: {code:?}, modifiers: {modifiers:?}"
            )),
        }
    }
}
