use std::{collections::HashMap, sync::Arc};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Layout};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    action::Action,
    connector_worker::ConnectorWorker,
    connectors::ConnectorCommands,
    keybindings_trie::{ConnectorEvents, KeyBindingsTrie},
    mode::Mode,
    settings::{ConfigSource, Settings, get_config_dir},
    terminal::{self, Event, Tui},
    ui::{self, ActionResult, Drawable},
};

pub struct App {
    should_quit: bool,
    settings: Settings,
    connectors: HashMap<String, mpsc::Sender<ConnectorCommands>>,
    components: ui::Components,
    mode: Mode,
    tui: Tui,
}
impl App {
    pub fn new() -> Result<Self> {
        let config_file_path = get_config_dir().join("config.toml");
        let config_source = ConfigSource::File(config_file_path);
        let settings = Settings::new(config_source)?;
        let mode = Mode::default();

        let tui = Tui::new()?;
        Ok(Self {
            should_quit: false,
            components: ui::Components::new(),
            settings,
            connectors: HashMap::new(),
            mode,
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
            match self.mode {
                Mode::AddTorrent => {
                    self.components
                        .add_torrent
                        .draw(buffer, content_area, &self.settings);
                }
                _ => {
                    self.components
                        .torrent_list
                        .draw(buffer, content_area, &self.settings);
                    self.components
                        .notifications
                        .draw(buffer, notification_area, &self.settings);
                }
            };
            self.components
                .which_key
                .draw(buffer, content_area, &self.settings);
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

    fn notify(&mut self, n: ConnectorEvents) {
        self.components.notifications.notify(n);
    }

    async fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let result = match self.mode {
            Mode::TorrentList => {
                self.components
                    .torrent_list
                    .handle_key_events(key_event, &mut self.settings, &mut self.connectors)
                    .await?
            }
            Mode::AddTorrent => {
                self.components
                    .add_torrent
                    .handle_key_events(key_event, &mut self.settings, &mut self.connectors)
                    .await?
            }
        };
        match result {
            ActionResult::Handled => {}
            ActionResult::Unhandled => {
                if let Some(action) = self
                    .settings
                    .keybindings
                    .action(Mode::default(), key_event)?
                {
                    match action {
                        Action::Quit => self.should_quit = true,
                        Action::AddTorrent => self.mode = Mode::AddTorrent,
                        Action::Help => self.show_help()?,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn show_help(&mut self) -> Result<()> {
        if let Some(node) = self.settings.keybindings.get_node(&self.mode) {
            self.components.which_key.update(node);
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
