use std::io::stdout;

use color_eyre::eyre::{self, Result};
use ratatui::crossterm::{
    self, cursor,
    event::{self, Event as CrosstermEvent, KeyEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Clone)]
pub enum Event {
    Key(KeyEvent),
}

pub struct Tui {
    pub event_rx: Receiver<Event>,
    pub event_tx: Sender<Event>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(32);
        Ok(Self { event_rx, event_tx })
    }

    pub fn run(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            Self::event_loop(event_tx).await?;
            Ok::<(), eyre::Error>(())
        });
        Ok(())
    }

    async fn event_loop(event_tx: Sender<Event>) -> Result<()> {
        loop {
            if let CrosstermEvent::Key(key) = event::read()? {
                event_tx.send(Event::Key(key)).await?;
            }
        }
    }

    pub fn exit(&mut self) -> Result<()> {
        crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }
}
