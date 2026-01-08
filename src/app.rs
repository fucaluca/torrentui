use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    actions::Action,
    app_state::{AppState, ConnectorEvents},
    connector_worker::ConnectorWorker,
    key_mode::KeyMode,
    settings::Settings,
    tui::{self, Event, Tui},
};

pub struct App<'a> {
    should_quit: bool,
    app_state: AppState<'a>,
    settings: &'a Settings,
}
impl<'a> App<'a> {
    pub fn new(settings: &'a Settings) -> Result<Self> {
        let app_state = AppState::builder(settings)
            .key_mode(KeyMode::default())
            .build()?;

        Ok(Self {
            should_quit: false,
            app_state,
            settings,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;
        tui.run(self.settings)?;
        let (connector_events_tx, mut connector_events_rx) = mpsc::channel(10);
        let cancellation_token = tui.cancellation_token.clone();
        self.run_workers(connector_events_tx, cancellation_token)
            .await;

        loop {
            tokio::select! {
                Some(connector_event) = connector_events_rx.recv() => {
                    self.handle_connector_event(connector_event);
                },
                Some(tui_event) = tui.event_rx.recv() => {
                    self.handle_tui_events(tui_event).await?;
                }
            }

            if self.should_quit {
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    pub async fn run_workers(
        &mut self,
        connector_events_tx: mpsc::Sender<ConnectorEvents>,
        cancellation_token: CancellationToken,
    ) {
        for (connector_name, connector) in self.settings.connectors.0.iter() {
            let (command_tx, command_rx) = mpsc::channel(10);
            self.app_state
                .add_commands_tx(connector_name.into(), command_tx);
            let worker = ConnectorWorker::new(Arc::clone(connector), cancellation_token.clone());
            worker.run(command_rx, connector_events_tx.clone()).await;
        }
    }

    pub fn handle_connector_event(&mut self, event: ConnectorEvents) {
        match event {
            ConnectorEvents::UpdateTorrentList(connector_name, torrent_list) => {
                self.app_state
                    .update_torrent_list(connector_name, torrent_list);
            }
            ConnectorEvents::Error(e) => eprintln!("{e:#?}"),
            _ => {
                println!("TODO: implement notifications")
            }
        }
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

    async fn handle_tui_events(&mut self, event: tui::Event) -> Result<()> {
        match event {
            Event::Key(key_event) => self.handle_key_events(key_event).await?,
            Event::UpdateTorrentList => {}
        }
        Ok(())
    }
}
