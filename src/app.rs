use std::{collections::HashMap, sync::Arc};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Layout};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    action::Action,
    connector_worker::ConnectorWorker,
    connectors::{ConnectorCommands, ConnectorEvents},
    mode::{AddTorrentMode, KeyMode},
    settings::{ConfigSource, Settings, get_config_dir},
    terminal::{self, Event, Tui},
    ui::{self, Drawable, notifications::Notification},
};

#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum CurrentScreen {
    #[default]
    TorrentList,
    AddTorrent(AddTorrentMode),
}

pub struct App {
    should_quit: bool,
    settings: Settings,
    connectors: HashMap<String, mpsc::Sender<ConnectorCommands>>,
    components: ui::Components,
    current_screen: CurrentScreen,
    tui: Tui,
}
impl App {
    pub fn new() -> Result<Self> {
        let config_file_path = get_config_dir().join("config.toml");
        let config_source = ConfigSource::File(config_file_path);
        let settings = Settings::new(config_source)?;
        let mode = CurrentScreen::default();

        let tui = Tui::new()?;
        Ok(Self {
            should_quit: false,
            components: ui::Components::new(&settings),
            settings,
            connectors: HashMap::new(),
            current_screen: mode,
            tui,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.tui.run()?;

        let (connector_events_tx, mut connector_events_rx) = mpsc::channel(10);

        let cancellation_token = self.tui.cancellation_token.clone();

        self.run_workers(connector_events_tx, cancellation_token)
            .await;

        loop {
            tokio::select! {
                Some(connector_event) = connector_events_rx.recv() => {
                    self.handle_connector_event(connector_event)?;
                    self.render()?;
                },
                Some(tui_event) = self.tui.event_rx.recv() => {
                    self.handle_tui_events(tui_event).await?;
                    self.render()?;
                }
            }

            if self.should_quit {
                break;
            }
        }
        self.tui.exit()?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        let main_area = Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]);

        self.tui.terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            let [content_area, notification_area] = main_area.areas(area);
            match self.current_screen {
                CurrentScreen::AddTorrent(_) => {
                    self.components
                        .torrent_list
                        .draw(buffer, content_area, &self.settings);
                    self.components
                        .add_torrent
                        .draw(buffer, content_area, &self.settings);
                    self.components
                        .notifications
                        .draw(buffer, notification_area, &self.settings);
                    self.components
                        .which_key
                        .draw(buffer, content_area, &self.settings);
                }
                _ => {
                    self.components
                        .torrent_list
                        .draw(buffer, content_area, &self.settings);
                    self.components
                        .notifications
                        .draw(buffer, notification_area, &self.settings);
                    self.components
                        .which_key
                        .draw(buffer, content_area, &self.settings);
                }
            };
        })?;
        Ok(())
    }

    pub async fn run_workers(
        &mut self,
        connector_events_tx: mpsc::Sender<ConnectorEvents>,
        cancellation_token: CancellationToken,
    ) {
        for (connector_name, connector) in self.settings.connectors.iter() {
            let (command_tx, command_rx) = mpsc::channel(10);
            self.connectors
                .insert(connector_name.to_string(), command_tx);
            let worker = ConnectorWorker::new(Arc::clone(connector), cancellation_token.clone());
            worker.run(command_rx, connector_events_tx.clone()).await;
        }
    }

    pub fn handle_connector_event(&mut self, event: ConnectorEvents) -> color_eyre::Result<()> {
        match event {
            ConnectorEvents::UpdateTorrentList(connector_name, torrent_list) => {
                self.components
                    .torrent_list
                    .insert(connector_name, torrent_list);
                self.components.torrent_list.update_table();
            }
            n => self.notify(n),
        };
        Ok(())
    }

    fn notify(&mut self, n: impl Into<Option<Notification>>) {
        self.components.notifications.notify(n);
    }

    async fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let result = match self.current_screen {
            CurrentScreen::TorrentList => {
                self.components
                    .torrent_list
                    .handle_key_events(key_event, &mut self.settings, &mut self.connectors)
                    .await
            }
            CurrentScreen::AddTorrent(_) => {
                self.components
                    .add_torrent
                    .handle_key_events(key_event, &mut self.settings, &mut self.connectors)
                    .await
            }
        };
        match result {
            Ok(actions) => {
                for action in actions.iter() {
                    match action {
                        Action::Quit => self.should_quit = true,
                        Action::AddTorrent => {
                            self.switch_screen(CurrentScreen::AddTorrent(AddTorrentMode::Input))
                        }
                        // Action::AddTorrent => {
                        //     self.switch_screen(CurrentScreen::AddTorrent(AddTorrentMode::default()))
                        // }
                        Action::Help(mode) => self.show_help_on_current_screen(Some(mode))?,
                        Action::Next => {
                            if self.settings.show_help_auto {
                                self.show_help_on_current_screen(None)?;
                            }
                        }
                        Action::Escape => {
                            if self.components.which_key.is_hidden() {
                                self.current_screen = CurrentScreen::default();
                            } else {
                                self.components.which_key.hide();
                            }
                        }
                        _ => self.components.which_key.hide(),
                    }
                }
            }
            Err(e) => self.notify(e),
        }
        Ok(())
    }

    fn switch_screen(&mut self, screen: CurrentScreen) {
        self.components.which_key.hide();
        self.current_screen = screen;
    }

    fn show_help_on_current_screen(
        &mut self,
        current_screen: Option<&CurrentScreen>,
    ) -> Result<()> {
        let key_mode = KeyMode::from(current_screen.unwrap_or(&self.current_screen));
        self.show_help(&key_mode)?;
        Ok(())
    }

    fn show_help(&mut self, key_mode: &KeyMode) -> Result<()> {
        if let Some(node) = self.settings.keybindings.get_current_node(key_mode) {
            self.components.which_key.show(node);
        }
        Ok(())
    }

    async fn handle_tui_events(&mut self, event: terminal::Event) -> Result<()> {
        match event {
            Event::Key(key_event) => self.handle_key_events(key_event).await?,
        }
        Ok(())
    }
}
