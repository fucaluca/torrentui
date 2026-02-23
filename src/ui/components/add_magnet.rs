use std::{collections::HashMap, sync::Arc};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Row, StatefulWidget, Table, TableState,
        Widget,
    },
};
use snafu::ResultExt;
use tokio::sync::mpsc;

use crate::{
    app::CurrentScreen,
    connectors::{ConnectorCommands, ConnectorName},
    domain::{
        action::{
            Action, ActionError, ClipboardInitSnafu, CreateMagnetSnafu, GetActionFailedSnafu,
            GetFromClipboardSnafu, SendSnafu,
        },
        modes::{AddMagnetMode, KeyMode},
        torrent::{Magnet, Source},
    },
    settings::{Settings, styles::StyleMode},
    ui::{
        Drawable,
        assets::{self, Symbols},
    },
};

struct Connector {
    name: ConnectorName,
    selected: bool,
}

impl Connector {
    fn toggle_selected(&mut self) {
        self.selected = !self.selected
    }
}

#[derive(Default)]
pub struct AddMagnet {
    table_state: TableState,
    text_area: String,
    torrent_source: Option<Source>,
    connectors: Vec<Connector>,
    mode: AddMagnetMode,
}

impl Drawable for AddMagnet {
    fn draw(
        &mut self,
        buf: &mut ratatui::prelude::Buffer,
        area: ratatui::prelude::Rect,
        settings: &Settings,
    ) {
        let borders_height: u16 = 2;
        let text_area_height: u16 = 1;
        let divider_height: u16 = 1;
        let connectors_count: u16 = settings.connectors.len().min(area.height as usize) as u16;
        let height: u16 = borders_height + text_area_height + divider_height + connectors_count;
        let width: u16 = 80;
        let centered_area =
            self.centered_rect(width.min(area.width), height.min(area.height), area);

        let style = settings
            .styles
            .get_style(&StyleMode::AddTorrent(self.mode), "border");

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(style);

        let inner_area = block.inner(centered_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(text_area_height),
                Constraint::Length(divider_height),
                Constraint::Min(connectors_count),
            ])
            .split(inner_area);
        let input_area = chunks[0];
        let divider_area = chunks[1];
        let connectors_area = chunks[2];

        let text_area_style = settings
            .styles
            .get_style(&StyleMode::AddTorrent(self.mode), "input");

        let table = self.build_table(settings);

        Clear.render(centered_area, buf);
        block.render(centered_area, buf);
        Paragraph::new(self.text_area.clone())
            .style(text_area_style)
            .render(input_area, buf);
        Line::from(Symbols::ROW_DIVIDER.repeat(divider_area.width as usize))
            .style(text_area_style)
            .render(divider_area, buf);
        StatefulWidget::render(table, connectors_area, buf, &mut self.table_state)
    }
}

impl AddMagnet {
    pub fn new(settings: &Settings) -> Self {
        let connectors = settings
            .connectors
            .iter()
            .map(|(name, connector)| Connector {
                name: Arc::clone(name),
                selected: *connector.connector.selected(),
            })
            .collect::<Vec<Connector>>();
        Self {
            connectors,
            ..Default::default()
        }
    }

    pub async fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        settings: &mut Settings,
        connectors: &mut HashMap<String, mpsc::Sender<ConnectorCommands>>,
    ) -> Result<Option<Action>, ActionError> {
        use AddMagnetMode::*;
        match self.mode {
            Input => self.edit_mode_action(key_event, settings, connectors).await,
            Connectors => {
                self.connector_mode_action(key_event, settings, connectors)
                    .await
            }
        }
    }

    async fn edit_mode_action(
        &mut self,
        key_event: KeyEvent,
        settings: &mut Settings,
        connectors: &mut HashMap<String, mpsc::Sender<ConnectorCommands>>,
    ) -> Result<Option<Action>, ActionError> {
        let action = settings
            .keybindings
            .action(KeyMode::AddTorrent(AddMagnetMode::Input), key_event)
            .context(GetActionFailedSnafu)?;

        if action.is_none() {
            return Ok(None);
        }

        use Action::*;
        match action.unwrap() {
            Paste => {
                let text = self.get_from_clipboard()?;
                self.text_area = text.clone();
                let magnet = Magnet::new(text).context(CreateMagnetSnafu)?;
                self.torrent_source = Some(Source::Magnet(magnet));
                Ok(None)
            }
            Backspace => {
                self.text_area.clear();
                self.torrent_source = None;
                Ok(None)
            }
            Enter => {
                if self.torrent_source.is_none() {
                    return Ok(None);
                }
                if connectors.len() == 1 {
                    self.send_new_torrent(connectors).await?;
                    Ok(Some(Action::DefaultScreen))
                } else {
                    self.mode = AddMagnetMode::Connectors;
                    Ok(None)
                }
            }
            Switch => {
                self.mode.toggle();
                Ok(None)
            }
            Help(_) => Ok(Some(Help(CurrentScreen::AddTorrent(AddMagnetMode::Input)))),
            action => Ok(Some(action)),
        }
    }

    async fn connector_mode_action(
        &mut self,
        key_event: KeyEvent,
        settings: &mut Settings,
        connectors: &mut HashMap<String, mpsc::Sender<ConnectorCommands>>,
    ) -> Result<Option<Action>, ActionError> {
        let action = settings
            .keybindings
            .action(KeyMode::AddTorrent(AddMagnetMode::Connectors), key_event)
            .context(GetActionFailedSnafu)?;
        if action.is_none() {
            return Ok(None);
        }
        use Action::*;
        match action.unwrap() {
            Send => {
                self.send_new_torrent(connectors).await?;
                Ok(Some(Action::DefaultScreen))
            }
            Switch => {
                self.mode.toggle();
                Ok(None)
            }
            Toggle => {
                self.toggle_connector();
                Ok(None)
            }
            Up => {
                self.table_state.select_previous();
                Ok(None)
            }
            Down => {
                self.table_state.select_next();
                Ok(None)
            }
            Help(_) => Ok(Some(Help(CurrentScreen::AddTorrent(
                AddMagnetMode::Connectors,
            )))),
            action => Ok(Some(action)),
        }
    }

    async fn send_new_torrent(
        &self,
        connectors: &mut HashMap<String, mpsc::Sender<ConnectorCommands>>,
    ) -> Result<(), ActionError> {
        let source = if let Some(source) = &self.torrent_source {
            source.clone()
        } else {
            let magnet = Magnet::new(self.text_area.clone()).context(CreateMagnetSnafu)?;
            Source::Magnet(magnet)
        };
        let command = ConnectorCommands::Add(source);
        self.send(connectors, command).await?;
        Ok(())
    }

    async fn send(
        &self,
        connectors: &mut HashMap<String, mpsc::Sender<ConnectorCommands>>,
        command: ConnectorCommands,
    ) -> Result<(), ActionError> {
        for con in &self.connectors {
            if con.selected
                && let Some(connector) = connectors.get_mut(&con.name.to_string())
            {
                connector.send(command.clone()).await.context(SendSnafu)?
            }
        }
        Ok(())
    }

    fn toggle_connector(&mut self) {
        if let Some(selected_connector_idx) = self.table_state.selected()
            && let Some(connector) = self.connectors.get_mut(selected_connector_idx)
        {
            connector.toggle_selected();
        }
    }

    pub fn update(&mut self, settings: &Settings) {
        if settings.auto_insert_torrent {
            self.try_insert_magnet_silent();
        }
    }

    fn try_insert_magnet_silent(&mut self) {
        match self.get_from_clipboard() {
            Ok(maybe_magnet) => match Magnet::new(maybe_magnet.clone()) {
                Ok(magnet) => {
                    self.text_area = maybe_magnet;
                    self.torrent_source = Some(Source::Magnet(magnet));
                }
                Err(_) => { /* nothing to do */ }
            },
            Err(_) => { /* nothing to do */ }
        }
    }

    fn get_from_clipboard(&self) -> Result<String, ActionError> {
        let mut clipboard = arboard::Clipboard::new().context(ClipboardInitSnafu)?;
        Ok(clipboard
            .get_text()
            .context(GetFromClipboardSnafu)?
            .replace(['\n', '\r'], ""))
    }

    fn build_table(&mut self, settings: &Settings) -> Table<'static> {
        let rows: Vec<Row<'_>> = self
            .connectors
            .iter()
            .map(|connector| {
                let style;
                let icon;
                if connector.selected {
                    style = settings
                        .styles
                        .get_style(&StyleMode::AddTorrent(self.mode), "selected_connector");
                    icon = assets::Icons::CHECKED;
                } else {
                    style = settings
                        .styles
                        .get_style(&StyleMode::AddTorrent(self.mode), "default");
                    icon = assets::Icons::UNCHECKED;
                }
                Row::new(vec![
                    Line::from(icon.to_string()),
                    Line::from(connector.name.to_string()),
                ])
                .style(style)
            })
            .collect();
        let widths = [Constraint::Length(2), Constraint::Fill(1)];
        let table_style = settings
            .styles
            .get_style(&StyleMode::AddTorrent(self.mode), "default");

        let highlight_style = settings
            .styles
            .get_style(&StyleMode::AddTorrent(self.mode), "connectors_highlight");
        Table::new(rows, widths)
            .column_spacing(1)
            .style(table_style)
            .row_highlight_style(highlight_style)
    }

    fn centered_rect(&self, width: u16, height: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(height),
                Constraint::Min(0),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(width),
                Constraint::Min(0),
            ])
            .split(popup_layout[1])[1]
    }
}
