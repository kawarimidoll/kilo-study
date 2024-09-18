use super::Position;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{
    Attribute::{Reset, Reverse},
    Print,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
    EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use crossterm::{queue, Command};
use std::io::{stdout, Error, Write};

#[derive(Copy, Clone, Default)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

/// Represents the Terminal.
/// Edge Case for platforms where `usize` < `u16`:
/// Regardless of the actual size of the Terminal, this representation only spans ofer as most
/// `usize::MAX` or `u16::size` rows / colmuns, whichever is smaler.
/// Each size returned truncates to min(`usize::MAX`, `u16::size`)
/// And should you attempt to set the caret out of those bounds, it will also be truncated.
pub struct Terminal;

impl Terminal {
    pub fn terminate() -> Result<(), Error> {
        Self::leave_alternate_screen()?;
        Self::enable_line_wrap()?;
        Self::show_caret()?;
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::disable_line_wrap()?;
        Self::clear_screen()?;
        Self::execute()?;
        Ok(())
    }
    pub fn enter_alternate_screen() -> Result<(), Error> {
        Self::queue_command(EnterAlternateScreen)?;
        Ok(())
    }
    pub fn leave_alternate_screen() -> Result<(), Error> {
        Self::queue_command(LeaveAlternateScreen)?;
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
    pub fn disable_line_wrap() -> Result<(), Error> {
        Self::queue_command(DisableLineWrap)?;
        Ok(())
    }
    pub fn enable_line_wrap() -> Result<(), Error> {
        Self::queue_command(EnableLineWrap)?;
        Ok(())
    }
    pub fn set_title(title: &str) -> Result<(), Error> {
        Self::queue_command(SetTitle(title))?;
        Ok(())
    }
    /// Moves the caret to the given Position.
    /// # Arguments
    /// * `Position` - the `Poisition` to move the caret to. Will be truncated to `u16::MAX` if
    ///   bitter.
    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))?;
        Ok(())
    }
    pub fn hide_caret() -> Result<(), Error> {
        Self::queue_command(Hide)?;
        Ok(())
    }
    pub fn show_caret() -> Result<(), Error> {
        Self::queue_command(Show)?;
        Ok(())
    }
    pub fn print(string: &str) -> Result<(), Error> {
        Self::queue_command(Print(string))?;
        Ok(())
    }
    pub fn print_row(row: usize, line_text: &str) -> Result<(), Error> {
        Self::move_caret_to(Position { col: 0, row })?;
        Self::clear_line()?;
        Self::print(line_text)?;
        Ok(())
    }
    pub fn print_invert_row(row: usize, line_text: &str) -> Result<(), Error> {
        let width = Self::size()?.width;
        // :width$ -> pad to width
        // .width$ -> truncate to width
        Self::print_row(row, &format!("{Reverse}{line_text:width$.width$}{Reset}"))
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
