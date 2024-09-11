#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]

use std::io::Error;
mod editor;
use editor::Editor;

fn main() -> Result<(), Error> {
    let mut editor = Editor::default();
    let args: Vec<String> = std::env::args().collect();
    if let Some(first) = args.get(1) {
        editor.load(String::from(first))?;
    }
    editor.run();
    Ok(())
}
