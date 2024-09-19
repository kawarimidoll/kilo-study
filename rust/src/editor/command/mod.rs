use crate::prelude::Size;
use crossterm::event::Event::{self, Key};
use std::convert::TryFrom;
mod edit;
mod movecommand;
mod system;
pub use edit::Edit;
pub use movecommand::Move;
pub use system::System;

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
