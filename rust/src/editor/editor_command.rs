use super::terminal::Size;
use crossterm::event::{
    Event::{self, Key},
    KeyCode::{Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    KeyEvent, KeyModifiers,
};
use std::convert::TryFrom;

pub enum Direction {
    Down,
    End,
    Home,
    Left,
    PageDown,
    PageUp,
    Right,
    Up,
}

pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Char(char),
    Quit,
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (Char('q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => Ok(Self::Char(c)),
                (Down, _) | (Char('n'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::Down)),
                (End, _) | (Char('e'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::End)),
                (Home, _) | (Char('a'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::Home)),
                (Left, _) | (Char('b'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::Left)),
                (PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (Right, _) | (Char('f'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::Right)),
                (Up, _) | (Char('p'), KeyModifiers::CONTROL) => Ok(Self::Move(Direction::Up)),
                _ => Err(format!(
                    "Unrecognized key: {code:?}, modifiers: {modifiers:?}"
                )),
            },
            #[allow(clippy::as_conversions)]
            Event::Resize(width16, height16) => Ok(Self::Resize(Size {
                width: width16 as usize,
                height: height16 as usize,
            })),
            _ => Err(format!("Unrecognized event: {event:?}")),
        }
    }
}
