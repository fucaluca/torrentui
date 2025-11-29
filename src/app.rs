use color_eyre::eyre::Result;
use ratatui::crossterm::event::KeyEvent;

use crate::tui::{Event, Tui};

#[derive(Default)]
pub struct App {
    should_quit: bool,
}
impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.run()?;
        loop {
            self.handle_events(&mut tui).await?;
            if self.should_quit {
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.event_rx.recv().await else {
            return Ok(());
        };
        // TODO: Добавить обработку других событий
        let Event::Key(key) = event;
        self.handle_key_events(key).await?;

        Ok(())
    }

    async fn handle_key_events(&self, _key: KeyEvent) -> Result<()> {
        todo!("handle key events")
    }
}
