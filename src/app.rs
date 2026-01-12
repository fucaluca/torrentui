use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    actions::Action,
    connector_worker::ConnectorWorker,
    key_mode::KeyMode,
    keybindings_trie::{ConnectorCommands, ConnectorEvents, KeyBindingsTrie},
    settings::Settings,
    terminal::{self, Event, Tui},
    torrent::TorrentInfo,
    ui::{self, Drawable},
};

pub struct App<'a> {
    should_quit: bool,
    keybindings_trie: KeyBindingsTrie<'a>,
    settings: &'a Settings,
    connectors: HashMap<String, mpsc::Sender<ConnectorCommands>>,
    components: ui::Components,
    tui: Tui,
}
impl<'a> App<'a> {
    pub fn new(settings: &'a Settings) -> Result<Self> {
        let keybindings_trie = KeyBindingsTrie::builder(&settings.keybindings)
            .key_mode(KeyMode::default())
            .build()?;

        let tui = Tui::new()?;
        Ok(Self {
            should_quit: false,
            keybindings_trie,
            settings,
            components: ui::Components::new(),
            connectors: HashMap::new(),
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
        self.tui.terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            if let Err(err) = self.components.torrent_list.draw(buffer, area) {
                panic!("{err:#?}");
            };
        })?;
        Ok(())
    }

    pub async fn run_workers(
        &mut self,
        connector_events_tx: mpsc::Sender<ConnectorEvents>,
        cancellation_token: CancellationToken,
    ) {
        for (connector_name, connector) in self.settings.connectors.0.iter() {
            let (command_tx, command_rx) = mpsc::channel(10);
            self.connectors.insert(connector_name.into(), command_tx);
            let worker = ConnectorWorker::new(Arc::clone(connector), cancellation_token.clone());
            worker.run(command_rx, connector_events_tx.clone()).await;
        }
    }

    pub fn handle_connector_event(&mut self, event: ConnectorEvents) -> color_eyre::Result<()> {
        match event {
            ConnectorEvents::UpdateTorrentList(connector_name, torrent_list) => {
                self.components
                    .torrent_list
                    .update_table(connector_name, torrent_list)?;
            }
            ConnectorEvents::Error(e) => eprintln!("{e:#?}"),
            _ => {
                println!("TODO: implement notifications")
            }
        };
        Ok(())
    }

    /* fn update_torrent_list(&mut self, connector_name) -> color_eyre::Result<()> {
        let action = UiAction::UpdateTorrentLinst;
        /* for component in self.components.iter_mut() {
            component.update(action.clone())?;
        } */
        Ok(())
    } */

    /* fn update_component(&mut self, action: UiAction) -> color_eyre::Result<()> {
        for component in self.components.iter_mut() {
            component.update(action)?;
        }
        Ok(())
    } */

    async fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        if let Some(action) = self.keybindings_trie.action(key_event) {
            match action {
                Action::Quit => self.should_quit = true,
                Action::AddTorrent => todo!(),
                Action::NoOp => {}
            }
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
