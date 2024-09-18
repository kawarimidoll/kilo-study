use buffer::Buffer;
use line::Line;
use std::cmp::min;
use std::io::Error;
mod buffer;
mod line;
mod location;
use super::{
    command::{Edit, Move},
    terminal::Terminal,
    ui_component::UIComponent,
    Col, Position, Row, Size, NAME, VERSION,
};
use location::Location;

const FILLCHAR_EOB: &str = "~";

struct SearchInfo {
    prev_location: Location,
    prev_scroll_offset: Position,
    query: Option<Line>,
}
#[derive(Default, Eq, PartialEq)]
enum SearchDirection {
    #[default]
    Forward,
    Backward,
}

#[derive(Default)]
pub struct View {
    pub buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    pub text_location: Location,
    scroll_offset: Position,
    search_info: Option<SearchInfo>,
}

impl View {
    pub fn handle_edit_command(&mut self, edit_command: Edit) {
        match edit_command {
            Edit::Insert(c) => self.insert(c),
            Edit::InsertNewLine => self.enter(),
            Edit::DeleteBackward => self.backspace(),
            Edit::Delete => self.delete(),
        }
    }
    pub fn save(&mut self) -> Result<(), Error> {
        self.buffer.save()
    }
    pub fn save_as(&mut self, filename: &str) -> Result<(), Error> {
        self.buffer.save_as(filename)
    }
    pub fn insert(&mut self, c: char) {
        if self.buffer.insert_char(c, self.text_location) {
            self.handle_move_command(Move::Right);
            self.needs_redraw = true;
        }
    }
    pub fn enter(&mut self) {
        if self.buffer.insert_newline(self.text_location) {
            self.handle_move_command(Move::Right);
            self.needs_redraw = true;
        }
    }
    pub fn backspace(&mut self) {
        let Location {
            grapheme_idx,
            line_idx,
        } = self.text_location;
        // out of bounds
        if grapheme_idx == 0 && line_idx == 0 {
            return;
        }
        self.handle_move_command(Move::Left);
        self.delete();
    }
    pub fn delete(&mut self) {
        if self.buffer.remove_char(self.text_location) {
            self.needs_redraw = true;
        }
    }
    pub fn enter_search(&mut self) {
        self.search_info = Some(SearchInfo {
            prev_location: self.text_location,
            prev_scroll_offset: self.scroll_offset,
            query: None,
        });
    }
    pub fn dismiss_search(&mut self) {
        if let Some(search_info) = &self.search_info {
            self.text_location = search_info.prev_location;
            self.scroll_offset = search_info.prev_scroll_offset;
            self.set_needs_redraw(true);
        }
        self.exit_search();
    }
    pub fn exit_search(&mut self) {
        self.search_info = None;
    }
    fn get_search_query(&self) -> Option<&Line> {
        let query = self
            .search_info
            .as_ref()
            .and_then(|search_info| search_info.query.as_ref());
        debug_assert!(query.is_some(), "Empty search query");
        query
    }
    pub fn search(&mut self, query: &str) {
        if let Some(search_info) = &mut self.search_info {
            search_info.query = Some(Line::from(query));
        }
        self.search_in_direction(self.text_location, SearchDirection::default());
    }
    fn search_in_direction(&mut self, from: Location, direction: SearchDirection) {
        let option_location = self.get_search_query().and_then(|query| {
            if query.is_empty() {
                None
            } else if direction == SearchDirection::Forward {
                self.buffer.search_forward(query, from)
            } else {
                self.buffer.search_backward(query, from)
            }
        });

        if let Some(location) = option_location {
            self.text_location = location;
            self.center_text_location();
        }
    }
    pub fn search_next(&mut self) {
        let step_right = self
            .get_search_query()
            .map_or(1, |query| min(query.grapheme_count(), 1));

        let location = Location {
            grapheme_idx: self.text_location.grapheme_idx.saturating_add(step_right),
            line_idx: self.text_location.line_idx,
        };
        self.search_in_direction(location, SearchDirection::Forward);
    }
    pub fn search_prev(&mut self) {
        self.search_in_direction(self.text_location, SearchDirection::Backward);
    }
    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        let buffer = Buffer::load(filename)?;
        self.buffer = buffer;
        self.needs_redraw = true;
        Ok(())
    }
    fn draw_welcome_message(&self) {
        let messages = vec![
            "A long time ago in a galaxy far, far away...".to_string(),
            String::default(),
            format!("{NAME} editor -- version {VERSION}"),
        ];
        if messages.len() > self.size.height {
            return;
        }

        // minus 1 for FILLCHAR_EOB
        let display_width = self.size.width.saturating_sub(1);
        let mut row = self.size.height.div_ceil(3);
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
    fn render_line(row: Row, line_text: &str) {
        let result = Terminal::print_row(row, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
    }

    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(&self.scroll_offset)
    }
    pub fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_idx;
        debug_assert!(row.saturating_sub(1) < self.buffer.height());
        let col = self
            .buffer
            .lines
            .get(row)
            .map_or(0, |line| line.width_until(self.text_location.grapheme_idx));
        Position { col, row }
    }

    pub fn get_line(&self, row: Row) -> Option<&Line> {
        self.buffer.lines.get(row)
    }

    pub fn handle_move_command(&mut self, move_command: Move) {
        // This match moves the position, but does not check for all boundaries.
        // The final boundary checking happens after the match statement.
        match move_command {
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::Up => self.move_up(1),
            Move::Down => self.move_down(1),
            Move::StartOfLine => self.move_to_start_of_line(),
            Move::EndOfLine => self.move_to_end_of_line(),
            Move::PageUp => self.move_up(self.size.height.saturating_sub(1)),
            Move::PageDown => self.move_down(self.size.height.saturating_sub(1)),
        };

        self.scroll_into_view();
    }
    fn move_up(&mut self, step: Row) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_sub(step);
        self.snap_to_valid_x();
        self.snap_to_valid_y();
    }
    fn move_down(&mut self, step: Row) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_add(step);
        self.snap_to_valid_x();
        self.snap_to_valid_y();
    }
    fn move_left(&mut self) {
        if self.text_location.grapheme_idx > 0 {
            self.text_location.grapheme_idx = self.text_location.grapheme_idx.saturating_sub(1);
        } else {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }
    fn move_right(&mut self) {
        let line_len = self
            .get_line(self.text_location.line_idx)
            .map_or(0, Line::grapheme_count);
        if self.text_location.grapheme_idx == line_len {
            self.move_to_start_of_line();
            self.move_down(1);
        } else {
            self.text_location.grapheme_idx = self.text_location.grapheme_idx.saturating_add(1);
        }
    }
    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_idx = 0;
    }
    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_idx = self
            .get_line(self.text_location.line_idx)
            .map_or(0, Line::grapheme_count);
    }
    fn snap_to_valid_x(&mut self) {
        self.text_location.grapheme_idx = min(
            self.text_location.grapheme_idx,
            self.get_line(self.text_location.line_idx)
                .map_or(0, Line::grapheme_count),
        );
    }
    fn snap_to_valid_y(&mut self) {
        self.text_location.line_idx = min(self.text_location.line_idx, self.buffer.height());
    }
    fn center_text_location(&mut self) {
        let Size { width, height } = self.size;
        let Position { row, col } = self.text_location_to_position();
        let mid_width = width.div_ceil(2);
        let mid_height = height.div_ceil(2);
        self.scroll_offset.col = col.saturating_sub(mid_width);
        self.scroll_offset.row = row.saturating_sub(mid_height);
        self.set_needs_redraw(true);
    }
    fn scroll_into_view(&mut self) {
        let Position { col, row } = self.text_location_to_position();
        self.scroll_horizontally(col);
        self.scroll_vertically(row);
    }
    fn scroll_horizontally(&mut self, to: Col) {
        let width = self.size.width;
        if to < self.scroll_offset.col {
            self.scroll_offset.col = to;
            self.needs_redraw = true;
        } else if to >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);
            self.needs_redraw = true;
        }
    }
    fn scroll_vertically(&mut self, to: Row) {
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
            let line_text = self.get_line(current_row.saturating_add(top)).map_or_else(
                || FILLCHAR_EOB.to_string(),
                |line| line.get_visible_graphemes(left..right),
            );
            Self::render_line(current_row, &line_text);
        }
        if self.buffer.is_empty() {
            self.draw_welcome_message();
        }
        Ok(())
    }
}
