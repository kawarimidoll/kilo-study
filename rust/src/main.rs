#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]
mod editor;
mod prelude;
use editor::Editor;

fn main() {
    Editor::new().unwrap().run();
}
