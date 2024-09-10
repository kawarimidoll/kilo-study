use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType};
use crossterm::{queue, Command};
use std::fmt::Display;
use std::io::{stdout, Error, Write};

#[derive(Copy, Clone)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}
#[derive(Copy, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// Represents the Terminal.
/// Edge Case for platforms where `usize` < `u16`:
/// Regardless of the actual size of the Terminal, this representation only spans ofer as most
/// `usize::MAX` or `u16::size` rows / colmuns, whichever is smaler.
/// Each size returned truncates to min(`usize::MAX`, `u16::size`)
/// And should you attempt to set the cursor out of those bounds, it will also be truncated.
pub struct Terminal;

impl Terminal {
    pub fn terminate() -> Result<(), Error> {
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(Position { x: 0, y: 0 })?;
        Self::execute()?;
        Ok(())
    }
    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All))?;
        Ok(())
    }
    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::UntilNewLine))?;
        Ok(())
    }
    /// Moves the cursor to the given Position.
    /// # Arguments
    /// * `Position` - the `Poisition` to move the cursor to. Will be truncated to `u16::MAX` if
    ///   bitter.
    pub fn move_cursor_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.x as u16, position.y as u16))?;
        Ok(())
    }
    pub fn hide_cursor() -> Result<(), Error> {
        Self::queue_command(Hide)?;
        Ok(())
    }
    pub fn show_cursor() -> Result<(), Error> {
        Self::queue_command(Show)?;
        Ok(())
    }
    pub fn print<T: Display>(string: T) -> Result<(), Error> {
        Self::queue_command(Print(string))?;
        Ok(())
    }
    /// Returns the current size of this Terminal.
    /// Edge Case for systems with `usize` < `u16`
    /// * A `Size` representing the terminal size. Any coordinate `z` truncated with `usize` < `z`
    ///   < `u16`
    pub fn size() -> Result<Size, Error> {
        let (width16, height16) = size()?;
        #[allow(clippy::as_conversions)]
        let width = width16 as usize;
        #[allow(clippy::as_conversions)]
        let height = height16 as usize;
        Ok(Size { width, height })
    }
    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }
    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }
}
