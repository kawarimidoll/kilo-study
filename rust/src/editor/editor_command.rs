use super::terminal::Size;
use crossterm::event::{
    Event::{self, Key},
    KeyCode::{
        Backspace, Char, Delete, Down, End, Enter, Esc, Home, Left, PageDown, PageUp, Right, Tab,
        Up,
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
#[derive(Copy, Clone)]
pub enum Command {
    Move(Move),
    Edit(Edit),
    System(System),
}

impl TryFrom<Event> for Command {
    type Error = String;
    #[allow(clippy::as_conversions)]
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Key(key_event) => Edit::try_from(key_event)
                .map(Command::Edit)
                .or_else(|_| Move::try_from(key_event).map(Command::Move))
                .or_else(|_| System::try_from(key_event).map(Command::System))
                .map_err(|_| format!("Unrecognized event: {event:?}")),
            Event::Resize(width16, height16) => Ok(Self::System(System::Resize(Size {
                width: width16 as usize,
                height: height16 as usize,
            }))),
            _ => Err(format!("Unrecognized event: {event:?}")),
        }
    }
}
