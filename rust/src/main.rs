mod editor;
use editor::Editor;

fn main() {
    let editor = Editor::default();
    editor.run();
    // above is same as below
    // Editor::run(&editor);
}
