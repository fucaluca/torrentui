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
    settings::Settings,
    terminal::{self, Event, Tui},
    ui::{self, Drawable, KeyEventResult},
};

pub struct App<'a> {
    should_quit: bool,
    keybindings_trie: KeyBindingsTrie<'a>,
    settings: &'a Settings,
    connectors: HashMap<String, mpsc::Sender<ConnectorCommands>>,
    components: ui::Components<'a>,
    mode: Mode,
    tui: Tui,
}
impl<'a> App<'a> {
    pub fn new(settings: &'a Settings) -> Result<Self> {
        let mode = Mode::default();
        let keybindings_trie = KeyBindingsTrie::builder(&settings.keybindings)
            .key_mode(mode)
            .build()?;

        let tui = Tui::new()?;
        Ok(Self {
            should_quit: false,
            components: ui::Components::new(settings),
            keybindings_trie,
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
                Mode::AddTorrent | Mode::Input => {
                    self.components
                        .add_torrent
                        .draw(buffer, content_area, self.settings);
                }
                _ => {
                    self.components
                        .torrent_list
                        .draw(buffer, content_area, self.settings);
                    self.components
                        .notifications
                        .draw(buffer, notification_area, self.settings);
                }
            };
            self.components
                .which_key
                .draw(buffer, content_area, self.settings);
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
            /* ConnectorEvents::Error(e) => eprintln!("{e:#?}"),
            _ => {
                println!("TODO: implement notifications")
            } */
        };
        Ok(())
    }

    fn notify(&mut self, n: ConnectorEvents) {
        self.components.notifications.notify(n);
    }
    async fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let result = self.components.add_torrent.key_event(key_event);
        match result {
            KeyEventResult::Consumed => {}
            KeyEventResult::Ignored => self.handle_command(key_event).await?,
        }
        Ok(())
    }

    async fn handle_command(&mut self, key_event: KeyEvent) -> Result<()> {
        if let Some(action) = self.keybindings_trie.action(key_event) {
            self.components.which_key.clear();
            match action {
                Action::Help | Action::NoOp => self.show_help(),
                Action::Quit => self.should_quit = true,
                Action::Input => self.components.add_torrent.enable_input(),
                Action::AddTorrent => {
                    self.mode = Mode::AddTorrent;
                    self.keybindings_trie.key_mode(self.mode)?;
                }
                Action::Escape => {
                    self.mode = Mode::default();
                    self.keybindings_trie.key_mode(self.mode)?;
                    self.components.which_key.clear();
                }

                a => {
                    self.components.notifications.on_user_interaction();
                    let command = self.components.torrent_list.action(a);

                    if let Some((connector_name, command)) = command
                        && let Some(connector) =
                            self.connectors.get_mut(&connector_name.to_string())
                    {
                        connector.send(command).await?;
                    }
                }
            }
        }
        Ok(())
    }

    /* async fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        if let Some(action) = self.keybindings_trie.action(key_event) {
            self.components.which_key.clear();
            match action {
                Action::Input => {
                    self.mode = Mode::Input;
                }
                Action::NoOp | Action::Help => self.show_help(),
                Action::Quit => self.should_quit = true,
                Action::Escape => {
                    self.mode = Mode::default();
                    self.keybindings_trie.key_mode(self.mode)?;
                    self.components.which_key.clear();
                }
                Action::AddTorrent => {
                    self.mode = Mode::AddTorrent;
                    self.keybindings_trie.key_mode(self.mode)?;
                }
                a => {
                    self.components.notifications.on_user_interaction();
                    let command = self.components.torrent_list.action(a);

                    if let Some((connector_name, command)) = command
                        && let Some(connector) =
                            self.connectors.get_mut(&connector_name.to_string())
                    {
                        connector.send(command).await?;
                    }

                    let command = self.components.add_torrent.action(a);

                    if let Some((connector_name, command)) = command
                        && let Some(connector) =
                            self.connectors.get_mut(&connector_name.to_string())
                    {
                        connector.send(command).await?;
                    }
                }
            };
        } else {
            self.components.add_torrent.input(key_event.code);
            self.components.which_key.clear();
        }

        Ok(())
    } */

    fn show_help(&mut self) {
        self.components
            .which_key
            .update(self.keybindings_trie.keybindings);
    }

    async fn handle_tui_events(&mut self, event: terminal::Event) -> Result<()> {
        match event {
            Event::Key(key_event) => self.handle_key_events(key_event).await?,
        }
        Ok(())
    }
}
