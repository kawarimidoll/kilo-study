use buffer::Buffer;
use line::Line;
use std::cmp::min;
use std::io::Error;
mod buffer;
mod line;
mod location;
use super::{
    editor_command::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
    ui_component::UIComponent,
    NAME, VERSION,
};
use location::Location;

const FILLCHAR_EOB: &str = "~";

#[derive(Default)]
pub struct View {
    pub buffer: Buffer,
    pub needs_redraw: bool,
    pub size: Size,
    pub location: Location,
    pub scroll_offset: Position,
}

impl View {
    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_text_location(direction),
            EditorCommand::Char(c) => self.insert(c),
            EditorCommand::Enter => self.enter(),
            EditorCommand::Backspace => self.backspace(),
            EditorCommand::Delete => self.delete(),
            // other commands are already handled by the editor
            _=> {}
        }
    }
    pub fn save(&mut self) -> Result<(), Error> {
        self.buffer.save()
    }
    pub fn insert(&mut self, c: char) {
        if self.buffer.insert_char(c, self.location) {
            self.move_text_location(Direction::Right);
            self.needs_redraw = true;
        }
    }
    pub fn enter(&mut self) {
        if self.buffer.insert_newline(self.location) {
            self.move_text_location(Direction::Right);
            self.needs_redraw = true;
        }
    }
    pub fn backspace(&mut self) {
        let Location { x, y } = self.location;
        // out of bounds
        if x == 0 && y == 0 {
            return;
        }
        self.move_text_location(Direction::Left);
        self.delete();
    }
    pub fn delete(&mut self) {
        if self.buffer.remove_char(self.location) {
            self.needs_redraw = true;
        }
    }
    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        let buffer = Buffer::load(filename)?;
        self.buffer = buffer;
        self.needs_redraw = true;
        Ok(())
    }
    fn draw_welcome_message(&self) {
        let messages = vec![
            "Welcome!".to_string(),
            format!("{NAME} editor -- version {VERSION}"),
        ];
        if messages.len() > self.size.height {
            return;
        }
        let display_width = self.size.width.saturating_sub(1);

        // we don't care if our welcome message is put *exactly* in the middle.
        #[allow(clippy::integer_division)]
        let mut row = self.size.height / 3;
        for mut message in messages {
            if display_width < message.len() {
                Self::render_line(row, FILLCHAR_EOB);
            } else {
                message = format!("{FILLCHAR_EOB:<1}{message:^display_width$}");
                message.truncate(self.size.width.saturating_sub(1));
                Self::render_line(row, &message);
            }
            row = row.saturating_add(1);
        }
    }
    fn render_line(row: usize, line_text: &str) {
        let result = Terminal::print_row(row, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(&self.scroll_offset)
    }
    pub fn text_location_to_position(&self) -> Position {
        let row = self.location.y;
        let col = self
            .buffer
            .lines
            .get(row)
            .map_or(0, |line| line.width_until(self.location.x));
        Position { col, row }
    }

    pub fn get_line(&self, row: usize) -> Option<&Line> {
        self.buffer.lines.get(row)
    }

    pub fn move_text_location(&mut self, direction: Direction) {
        // This match moves the position, but does not check for all boundaries.
        // The final boundary checking happens after the match statement.
        match direction {
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Home => self.move_to_start_of_line(),
            Direction::End => self.move_to_end_of_line(),
            Direction::PageUp => self.move_up(self.size.height.saturating_sub(1)),
            Direction::PageDown => self.move_down(self.size.height.saturating_sub(1)),
        };

        self.scroll_into_view();
    }
    fn move_up(&mut self, step: usize) {
        self.location.y = self.location.y.saturating_sub(step);
        self.snap_to_valid_x();
        self.snap_to_valid_y();
    }
    fn move_down(&mut self, step: usize) {
        self.location.y = self.location.y.saturating_add(step);
        self.snap_to_valid_x();
        self.snap_to_valid_y();
    }
    fn move_left(&mut self) {
        if self.location.x > 0 {
            self.location.x = self.location.x.saturating_sub(1);
        } else {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }
    fn move_right(&mut self) {
        let line_len = self.get_line(self.location.y).map_or(0, Line::len);
        if self.location.x == line_len {
            self.move_to_start_of_line();
            self.move_down(1);
        } else {
            self.location.x = self.location.x.saturating_add(1);
        }
    }
    fn move_to_start_of_line(&mut self) {
        self.location.x = 0;
    }
    fn move_to_end_of_line(&mut self) {
        self.location.x = self.get_line(self.location.y).map_or(0, Line::len);
    }
    fn snap_to_valid_x(&mut self) {
        self.location.x = min(
            self.location.x,
            self.get_line(self.location.y).map_or(0, Line::len),
        );
    }
    fn snap_to_valid_y(&mut self) {
        self.location.y = min(self.location.y, self.buffer.height());
    }
    fn scroll_into_view(&mut self) {
        let Position { col, row } = self.text_location_to_position();
        self.scroll_horizontally(col);
        self.scroll_vertically(row);
    }
    fn scroll_horizontally(&mut self, to: usize) {
        let width = self.size.width;
        if to < self.scroll_offset.col {
            self.scroll_offset.col = to;
            self.needs_redraw = true;
        } else if to >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);
            self.needs_redraw = true;
        }
    }
    fn scroll_vertically(&mut self, to: usize) {
        let height = self.size.height;
        if to < self.scroll_offset.row {
            self.scroll_offset.row = to;
            self.needs_redraw = true;
        } else if to >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = to.saturating_sub(height).saturating_add(1);
            self.needs_redraw = true;
        }
    }
}

impl UIComponent for View {
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    fn set_size(&mut self, to: Size) {
        self.size = to;
        self.scroll_into_view();
    }
    fn draw(&mut self, origin_y: usize) -> Result<(), Error> {
        let _ = Terminal::move_caret_to(Position::default());
        let Size { width, height } = self.size;
        let top = self.scroll_offset.row.saturating_sub(origin_y);
        let left = self.scroll_offset.col;
        let right = left.saturating_add(width);
        let end_y = origin_y.saturating_add(height);
        for current_row in origin_y..end_y {
            let line_text = self
                .get_line(current_row.saturating_add(top))
                .map_or_else(|| FILLCHAR_EOB.to_string(), |line| line.get(left..right));
            Self::render_line(current_row, &line_text);
        }
        if self.buffer.is_empty() {
            self.draw_welcome_message();
        }
        Ok(())
    }
}
