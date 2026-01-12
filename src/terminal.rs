use std::io::{Stdout, stdout};

use ::crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use color_eyre::eyre::{self, Result};
use futures::StreamExt;
use ratatui::{
    crossterm::{
        self, cursor,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::CrosstermBackend,
};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub enum Event {
    Key(KeyEvent),
}

pub struct Tui {
    pub terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
    pub event_rx: Receiver<Event>,
    pub event_tx: Sender<Event>,
    pub cancellation_token: CancellationToken,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(32);
        Ok(Self {
            terminal: ratatui::Terminal::new(CrosstermBackend::new(stdout()))?,
            event_rx,
            event_tx,
            cancellation_token: CancellationToken::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        let event_tx = self.event_tx.clone();
        let cancellation_token = self.cancellation_token.clone();
        tokio::spawn(async move {
            Self::event_loop(event_tx, cancellation_token).await?;
            Ok::<(), eyre::Error>(())
        });
        Ok(())
    }

    async fn event_loop(
        event_tx: Sender<Event>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        let mut event_stream = EventStream::new();

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }

                crossterm_event = event_stream.next() => {
                    match crossterm_event {
                        Some(Ok(CrosstermEvent::Key(key))) if key.kind == KeyEventKind::Press => {
                            if event_tx.send(Event::Key(key)).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(_)) => continue,
                        Some(Err(_)) => break,
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }
    pub fn exit(&mut self) -> Result<()> {
        self.cancellation_token.cancel();
        crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }
}
