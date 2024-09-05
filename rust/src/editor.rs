use crossterm::event::{read, Event::Key, KeyCode::Char};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

pub struct Editor {}

impl Editor {
    pub fn default() -> Self {
        Editor {}
    }

    pub fn run(&self) {
        println!("kilo start");
        if let Err(err) = self.repl() {
            panic!("{err:#?}")
        }
        println!("kilo end");
    }

    pub fn repl(&self) -> Result<(), std::io::Error> {
        enable_raw_mode()?;

        loop {
            if let Key(event) = read()? {
                println!("{event:?}\r");
                if let Char(c) = event.code {
                    if c == 'q' {
                        break;
                    }
                }
            }
        }

        disable_raw_mode()?;
        Ok(())
    }
}
