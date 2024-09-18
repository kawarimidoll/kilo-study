use super::super::Size;
use std::io::Error;

pub trait UIComponent {
    // Marks this component as in need of redrawing or not
    fn set_needs_redraw(&mut self, value: bool);
    // Determines if a component needs to be rdrawn or not
    fn needs_redraw(&self) -> bool;
    // Sets the size of the component
    fn set_size(&mut self, to: Size);
    // Draws the component
    fn draw(&mut self, origin_y: usize) -> Result<(), Error>;

    fn resize(&mut self, to: Size) {
        self.set_size(to);
        self.set_needs_redraw(true);
    }

    // Renders the component
    fn render(&mut self, origin_y: usize) {
        if !self.needs_redraw() {
            return;
        }
        if let Err(err) = self.draw(origin_y) {
            #[cfg(debug_assertions)]
            panic!("Failed to render component: {err:?}");
            #[cfg(not(debug_assertions))]
            let _ = err;
        } else {
            self.set_needs_redraw(false);
        }
    }
}
