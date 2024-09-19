use crate::prelude::Size;
use crossterm::event::{
    KeyCode::{Char, Esc},
    KeyEvent, KeyModifiers,
};
use std::convert::TryFrom;

#[derive(Copy, Clone)]
pub enum System {
    Resize(Size),
    Quit,
    Save,
    Search,
    Dismiss,
}

impl TryFrom<KeyEvent> for System {
    type Error = String;
    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        match (code, modifiers) {
            (Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
            (Char('s'), KeyModifiers::CONTROL) => Ok(Self::Save),
            (Char('g'), KeyModifiers::CONTROL) => Ok(Self::Search),
            (Esc, KeyModifiers::NONE) => Ok(Self::Dismiss),
            _ => Err(format!(
                "Unrecognized key: {code:?}, modifiers: {modifiers:?}"
            )),
        }
    }
}
