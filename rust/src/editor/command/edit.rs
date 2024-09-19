use crossterm::event::{
    KeyCode::{Backspace, Char, Delete, Enter, Tab},
    KeyEvent, KeyModifiers,
};
use std::convert::TryFrom;

#[derive(Copy, Clone)]
pub enum Edit {
    Insert(char),
    InsertNewLine,
    DeleteBackward,
    Delete,
}

impl TryFrom<KeyEvent> for Edit {
    type Error = String;
    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        match (code, modifiers) {
            (Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => Ok(Self::Insert(c)),
            (Backspace, KeyModifiers::NONE) | (Char('h'), KeyModifiers::CONTROL) => {
                Ok(Self::DeleteBackward)
            }
            (Delete, KeyModifiers::NONE) | (Char('d'), KeyModifiers::CONTROL) => Ok(Self::Delete),
            (Enter, KeyModifiers::NONE) => Ok(Self::InsertNewLine),
            (Tab, KeyModifiers::NONE) => Ok(Self::Insert('\t')),
            _ => Err(format!(
                "Unrecognized key: {code:?}, modifiers: {modifiers:?}"
            )),
        }
    }
}
