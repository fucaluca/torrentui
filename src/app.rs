use color_eyre::eyre::Result;
use ratatui::crossterm::event::KeyEvent;

use crate::{
    settings::{ConfigSource, Settings, get_config_dir},
    tui::{Event, Tui},
};

pub struct App {
    should_quit: bool,
    settings: Settings,
}
impl App {
    pub fn new() -> Result<Self> {
        let config_file_path = get_config_dir().join("config.toml");
        let config_source = ConfigSource::File(config_file_path);
        let settings = Settings::new(config_source)?;
        Ok(Self {
            settings,
            should_quit: false,
        })
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
