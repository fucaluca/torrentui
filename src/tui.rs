use std::{io::stdout, time::Duration};

use ::crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use color_eyre::eyre::{self, Result};
use futures::StreamExt;
use ratatui::crossterm::{
    self, cursor,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::Interval,
};
use tokio_util::sync::CancellationToken;

use crate::settings::Settings;

#[derive(Clone)]
pub enum Event {
    Key(KeyEvent),
    UpdateTorrentList,
}

pub struct Tui {
    pub event_rx: Receiver<Event>,
    pub event_tx: Sender<Event>,
    pub cancellation_token: CancellationToken,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(32);
        Ok(Self {
            event_rx,
            event_tx,
            cancellation_token: CancellationToken::new(),
        })
    }

    pub fn run(&mut self, settings: &Settings) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        let event_tx = self.event_tx.clone();
        let cancellation_token = self.cancellation_token.clone();
        let mut update_torrent_list_interval = tokio::time::interval(Duration::from_secs(
            settings.update_torrent_list_interval as u64,
        ));
        tokio::spawn(async move {
            Self::event_loop(
                event_tx,
                cancellation_token,
                &mut update_torrent_list_interval,
            )
            .await?;
            Ok::<(), eyre::Error>(())
        });
        Ok(())
    }

    async fn event_loop(
        event_tx: Sender<Event>,
        cancellation_token: CancellationToken,
        update_torrent_list_interval: &mut Interval,
    ) -> Result<()> {
        let mut event_stream = EventStream::new();

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }

                _ = update_torrent_list_interval.tick() => {
                    if event_tx.send(Event::UpdateTorrentList).await.is_err() {
                        break;
                    }
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
