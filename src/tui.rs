use std::io::{Stdout, stdout};

use color_eyre::eyre::Result;
use ratatui::{
    Terminal,
    backend::CrosstermBackend as Backend,
    crossterm::{self, cursor, terminal::LeaveAlternateScreen},
};

pub struct Tui {
    pub terminal: Terminal<Backend<Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: Terminal::new(Backend::new(stdout()))?,
        })
    }

    pub fn exit(&mut self) -> Result<()> {
        crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
        crossterm::terminal::disable_raw_mode()?;

        Ok(())
    }
}
