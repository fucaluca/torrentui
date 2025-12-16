use color_eyre::eyre::Result;
use ratatui::crossterm::event::KeyEvent;

use crate::{
    actions::Action,
    app_state::AppState,
    key_mode::KeyMode,
    settings::Settings,
    tui::{Event, Tui},
};

pub struct App<'a> {
    should_quit: bool,
    app_state: AppState<'a>,
}
impl<'a> App<'a> {
    pub fn new(settings: &'a Settings) -> Result<Self> {
        let app_state = AppState::builder(settings)
            .key_mode(KeyMode::default())
            .build()?;

        Ok(Self {
            should_quit: false,
            app_state,
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

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        if let Some(event) = tui.event_rx.recv().await {
            let Event::Key(key_event) = event;
            self.handle_key_events(key_event).await?;
        }
        Ok(())
    }

    async fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        if let Some(action) = self.app_state.action(key_event) {
            match action {
                Action::Quit => self.should_quit = true,
                Action::AddTorrent => todo!(),
                Action::NoOp => {}
            }
        }

        Ok(())
    }
}
