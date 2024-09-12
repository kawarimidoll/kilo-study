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
                (Down, _) => Ok(Self::Move(Direction::Down)),
                (End, _) => Ok(Self::Move(Direction::End)),
                (Home, _) => Ok(Self::Move(Direction::Home)),
                (Left, _) => Ok(Self::Move(Direction::Left)),
                (PageDown, _) => Ok(Self::Move(Direction::PageDown)),
                (PageUp, _) => Ok(Self::Move(Direction::PageUp)),
                (Right, _) => Ok(Self::Move(Direction::Right)),
                (Up, _) => Ok(Self::Move(Direction::Up)),
                _ => Err(format!("Unrecognized key: {code:?}")),
            },
            Event::Resize(width16, height16) => {
                #[allow(clippy::as_conversions)]
                let width = width16 as usize;
                #[allow(clippy::as_conversions)]
                let height = height16 as usize;
                Ok(Self::Resize(Size { width, height }))
            }
            _ => Err(format!("Unrecognized event: {event:?}")),
        }
    }
}
