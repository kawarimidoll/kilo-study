use buffer::Buffer;
use std::cmp::min;
mod buffer;
mod line;
mod location;
use super::{
    editor_command::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};
use location::Location;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const FILLCHAR_EOB: &str = "~";

pub struct View {
    pub buffer: Buffer,
    pub needs_redraw: bool,
    pub size: Size,
    pub location: Location,
    pub scroll_offset: Location,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            location: Location::default(),
            scroll_offset: Location::default(),
        }
    }
}

impl View {
    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_point(&direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Quit => {}
        }
    }
    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.scroll_into_view();
        self.needs_redraw = true;
    }
    pub fn load(&mut self, filename: &str) {
        if let Ok(buffer) = Buffer::load(filename) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }
    fn draw_welcome_message(&self) {
        let messages = vec![
            "Welcome!".to_string(),
            format!("{NAME} editor -- version {VERSION}"),
        ];

        // we don't care if our welcome message is put *exactly* in the middle.
        #[allow(clippy::integer_division)]
        let mut row = self.size.height / 3;
        for mut message in messages {
            #[allow(clippy::integer_division)]
            let col = self.size.width.saturating_sub(message.len()) / 2;
            message = format!("{FILLCHAR_EOB}{}{}", " ".repeat(col), message);
            message.truncate(self.size.width.saturating_sub(1));
            Self::render_line(row, &message);
            row = row.saturating_add(1);
        }
    }
    fn render_line(row: usize, line_text: &str) {
        let result = Terminal::print_row(row, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }
    pub fn render_buffer(&self) {
        let top = self.scroll_offset.y;
        let left = self.scroll_offset.x;
        let right = left.saturating_add(self.size.width);
        for current_row in 0..self.size.height.saturating_sub(1) {
            let line_text =
                if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                    &line.get(left..right)
                } else {
                    FILLCHAR_EOB
                };
            Self::render_line(current_row, line_text);
        }
    }
    pub fn render(&mut self) {
        // render function
        if !self.needs_redraw || self.size.width == 0 || self.size.height == 0 {
            return;
        }
        let _ = Terminal::move_caret_to(Position::default());
        self.render_buffer();
        if self.buffer.is_empty() {
            self.draw_welcome_message();
        }
        // here comes status line
        Self::render_line(self.size.height.saturating_sub(1), "--------");
        self.needs_redraw = false;
    }

    pub fn get_position(&self) -> Position {
        self.location.subtract(&self.scroll_offset).into()
    }

    pub fn move_point(&mut self, direction: &Direction) {
        let Location { x, y } = self.location;
        let Size { height, .. } = Terminal::size().unwrap_or_default();
        match direction {
            Direction::Left => {
                if x == 0 && y == 0 {
                    // do nothing
                } else if x == 0 {
                    self.move_point(&Direction::Up);
                    self.move_point(&Direction::End);
                } else {
                    self.location.x = x.saturating_sub(1);
                }
            }
            Direction::Right => {
                let line_len = if let Some(line) = self.buffer.lines.get(y) {
                    line.len()
                } else {
                    0
                };
                if x == line_len {
                    self.move_point(&Direction::Down);
                    self.move_point(&Direction::Home);
                } else {
                    self.location.x = x.saturating_add(1);
                }
            }
            Direction::Up => self.location.y = y.saturating_sub(1),
            Direction::Down => {
                self.location.y = min(y.saturating_add(1), self.buffer.lines.len());
            }
            Direction::Home => self.location.x = 0,
            Direction::End => {
                let line_len = if let Some(line) = self.buffer.lines.get(y) {
                    line.len()
                } else {
                    0
                };
                self.location.x = line_len;
            }
            Direction::PageUp => {
                self.location.y = y.saturating_sub(height);
            }
            Direction::PageDown => {
                self.location.y = min(y.saturating_add(height), self.buffer.lines.len());
            }
        };

        // snap overrun to right
        let line_len = if let Some(line) = self.buffer.lines.get(self.location.y) {
            line.len()
        } else {
            0
        };
        self.location.x = min(self.location.x, line_len);

        self.scroll_into_view();
    }
    fn scroll_into_view(&mut self) {
        let Location { x, y } = self.location;
        let Size { width, height } = self.size;
        if x < self.scroll_offset.x {
            self.scroll_offset.x = x;
            self.needs_redraw = true;
        } else if x >= self.scroll_offset.x.saturating_add(width) {
            self.scroll_offset.x = x.saturating_sub(width).saturating_add(1);
            self.needs_redraw = true;
        }
        if y < self.scroll_offset.y {
            self.scroll_offset.y = y;
            self.needs_redraw = true;
        } else if y >= self.scroll_offset.y.saturating_add(height) {
            self.scroll_offset.y = y.saturating_sub(height).saturating_add(1);
            self.needs_redraw = true;
        }
    }
}
