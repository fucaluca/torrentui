use std::{collections::HashMap, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Row, StatefulWidget, Table, TableState,
        Widget,
    },
};
use snafu::{ResultExt, Snafu};
use tokio::sync::mpsc;

use crate::{
    action::{Action, CreateMagnetSnafu, SendSnafu},
    connectors::{ConnectorCommands, ConnectorName},
    settings::Settings,
    torrent::{Magnet, Source},
    ui::{ActionResult, Drawable, assets::Symbols},
};
use crate::{
    action::{ActionError, GetActionFailedSnafu},
    app::CurrentScreen,
    mode::{AddTorrentMode, KeyMode},
    settings::styles::StyleMode,
    ui::assets,
};

#[derive(Debug, Snafu)]
#[expect(unused)]
pub enum AddTorrentError {
    #[snafu(display("Failed to initialize clipboard"))]
    ClipboardError { source: arboard::Error },
}

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
pub struct AddTorrent {
    table_state: TableState,
    value: String,
    selection_start: Option<usize>,
    cursor_position: usize,
    insert_mode: bool,
    connectors: Vec<Connector>,
    mode: AddTorrentMode,
}

impl Drawable for AddTorrent {
    fn draw(
        &mut self,
        buf: &mut ratatui::prelude::Buffer,
        area: ratatui::prelude::Rect,
        settings: &Settings,
    ) {
        let centered_area = self.centered_rect(50, 10, area);
        Clear.render(centered_area, buf);
        let style = settings
            .styles
            .get_style(&StyleMode::AddTorrent(self.mode), "border");
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(style);

        let inner_area = block.inner(centered_area);
        block.render(centered_area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(3),
            ])
            .split(inner_area);

        let input_area = chunks[0];
        let offset = self
            .cursor_position
            .saturating_sub((input_area.width - 1) as usize);
        let mut display_text = self.value.chars().skip(offset).collect::<String>();

        if matches!(self.mode, AddTorrentMode::Input) {
            let input_style = if self.insert_mode {
                settings
                    .styles
                    .get_style(&StyleMode::AddTorrent(AddTorrentMode::Input), "insert_mode")
            } else {
                settings
                    .styles
                    .get_style(&StyleMode::AddTorrent(AddTorrentMode::Input), "default")
            };
            display_text.insert(
                self.cursor_position.saturating_sub(offset),
                assets::Symbols::CURSOR,
            );

            Paragraph::new(display_text)
                .style(input_style)
                .render(input_area, buf);
            self.highlight_selected(offset, input_area, buf, settings);
            Line::from(Symbols::ROW_DIVIDER.repeat(chunks[1].width as usize))
                .style(input_style)
                .render(chunks[1], buf);
        } else {
            let input_style = settings
                .styles
                .get_style(&StyleMode::AddTorrent(AddTorrentMode::Connectors), "input");
            Paragraph::new(display_text)
                .style(input_style)
                .render(input_area, buf);
            Line::from(Symbols::ROW_DIVIDER.repeat(chunks[1].width as usize))
                .style(input_style)
                .render(chunks[1], buf);
        }

        let table = self.build_table(settings);
        StatefulWidget::render(table, chunks[2], buf, &mut self.table_state)
    }
}

impl AddTorrent {
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

    fn highlight_selected(
        &mut self,
        skip: usize,
        area: Rect,
        buf: &mut Buffer,
        settings: &Settings,
    ) {
        if let Some(start) = self.selection_start {
            let start = start.saturating_sub(skip) as u16;
            let end = self.cursor_position.saturating_sub(skip) as u16;
            let highlight_style = settings
                .styles
                .get_style(&StyleMode::AddTorrent(self.mode), "input_highlight");

            for i in start.min(end + 1)..end.max(start + 1) {
                if let Some(cell) = buf.cell_mut((area.x + i, area.y)) {
                    cell.set_style(highlight_style);
                }
            }
        }
    }

    pub async fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        settings: &mut Settings,
        connectors: &mut HashMap<String, mpsc::Sender<ConnectorCommands>>,
    ) -> Result<ActionResult, ActionError> {
        use Action::*;
        let maybe_action = settings
            .keybindings
            .action(KeyMode::AddTorrent(self.mode), key_event)
            .context(GetActionFailedSnafu)?;
        if self.insert_mode {
            match maybe_action {
                Some(Escape) => {
                    self.insert_mode = false;
                    if self.selection_start.is_some() {
                        self.selection_start = None;
                    }
                    Ok(ActionResult::Handled)
                }
                Some(Backspace) => {
                    if let Some(start) = self.selection_start {
                        self.value = self
                            .value
                            .chars()
                            .enumerate()
                            .filter(|(i, _)| {
                                i < &start.min(self.cursor_position)
                                    || i >= &self.cursor_position.max(start)
                            })
                            .map(|s| s.1)
                            .collect();
                        self.cursor_position = start.min(self.cursor_position);
                        self.selection_start = None;
                    } else if self.value.chars().count() > 0 && self.cursor_position > 0 {
                        self.value.remove(self.cursor_position.saturating_sub(1));
                        self.cursor_position = self.cursor_position.saturating_sub(1);
                    }
                    Ok(ActionResult::Handled)
                }
                Some(SelectAll) => {
                    if self.selection_start.is_some() {
                        self.selection_start = Some(0);
                        self.cursor_position = self.value.chars().count();
                    } else {
                        self.selection_start = Some(0)
                    }
                    Ok(ActionResult::Handled)
                }
                Some(_) | None => {
                    self.insert(key_event.code);
                    Ok(ActionResult::Handled)
                }
            }
        } else if let Some(action) = maybe_action {
            match action {
                Escape => {
                    if self.selection_start.is_some() {
                        self.selection_start = None;
                        Ok(ActionResult::Handled)
                    } else {
                        Ok(ActionResult::Unhandled(action))
                    }
                }
                Input => {
                    self.insert_mode = true;
                    Ok(ActionResult::Handled)
                }
                Left => {
                    self.cursor_position = self.cursor_position.saturating_sub(1);
                    Ok(ActionResult::Handled)
                }
                Right => {
                    self.cursor_position = self.value.chars().count().min(self.cursor_position + 1);
                    Ok(ActionResult::Handled)
                }
                Select => {
                    if self.selection_start.is_some() {
                        self.selection_start = None;
                    } else {
                        self.selection_start = Some(self.cursor_position);
                    }
                    Ok(ActionResult::Handled)
                }
                SelectAll => {
                    if self.selection_start.is_some() {
                        self.selection_start = Some(0);
                        self.cursor_position = self.value.chars().count();
                    } else {
                        self.selection_start = Some(0)
                    }
                    Ok(ActionResult::Handled)
                }
                Backspace => {
                    if let Some(start) = self.selection_start {
                        self.value = self
                            .value
                            .chars()
                            .enumerate()
                            .filter(|(i, _)| {
                                i < &start.min(self.cursor_position)
                                    || i >= &self.cursor_position.max(start)
                            })
                            .map(|s| s.1)
                            .collect();
                        self.cursor_position = start.min(self.cursor_position);
                        self.selection_start = None;
                    } else if self.value.chars().count() > 0 && self.cursor_position > 0 {
                        self.value.remove(self.cursor_position.saturating_sub(1));
                        self.cursor_position = self.cursor_position.saturating_sub(1);
                    }
                    Ok(ActionResult::Handled)
                }
                Switch => {
                    self.mode.toggle();
                    Ok(ActionResult::Handled)
                }
                Up => {
                    self.table_state.select_previous();
                    Ok(ActionResult::Handled)
                }
                Down => {
                    self.table_state.select_next();
                    Ok(ActionResult::Handled)
                }
                Toggle => {
                    self.toggle_connector();
                    Ok(ActionResult::Handled)
                }
                Send => self.send_new_torrent(connectors).await,
                Help(_) => Ok(ActionResult::Unhandled(Help(CurrentScreen::AddTorrent(
                    self.mode,
                )))),

                _ => Ok(ActionResult::Unhandled(action)),
            }
        } else {
            Ok(ActionResult::Unhandled(Action::NoOp))
        }
    }

    async fn send_new_torrent(
        &self,
        connectors: &mut HashMap<String, mpsc::Sender<ConnectorCommands>>,
    ) -> Result<ActionResult, ActionError> {
        let magnet = Magnet::new(self.value.clone()).context(CreateMagnetSnafu)?;
        let source = Source::Magnet(magnet);
        let command = ConnectorCommands::Add(source);
        self.send(connectors, command).await
    }

    async fn send(
        &self,
        connectors: &mut HashMap<String, mpsc::Sender<ConnectorCommands>>,
        command: ConnectorCommands,
    ) -> Result<ActionResult, ActionError> {
        for con in &self.connectors {
            if con.selected
                && let Some(connector) = connectors.get_mut(&con.name.to_string())
            {
                connector.send(command.clone()).await.context(SendSnafu)?
            }
        }
        Ok(ActionResult::Unhandled(Action::Escape))
    }

    fn toggle_connector(&mut self) {
        if let Some(selected_connector_idx) = self.table_state.selected()
            && let Some(connector) = self.connectors.get_mut(selected_connector_idx)
        {
            connector.toggle_selected();
        }
    }

    pub fn insert(&mut self, key_code: KeyCode) {
        #[expect(clippy::single_match)]
        match key_code {
            KeyCode::Char(char) => {
                self.value.insert(self.cursor_position, char);
                self.cursor_position += 1;
            }
            _ => {}
        }
    }

    /* fn selected_connector(&mut self, settings: &mut Settings, idx: usize) {
        settings.connectors.get(key)
    } */

    fn build_table(&mut self, settings: &Settings) -> Table<'static> {
        let rows: Vec<Row<'_>> = self
            .connectors
            .iter()
            .map(|con| {
                let style;
                let icon;
                if con.selected {
                    style = settings
                        .styles
                        .get_style(&StyleMode::AddTorrent(self.mode), "selected_connector");
                    icon = "󰄵";
                } else {
                    style = settings
                        .styles
                        .get_style(&StyleMode::AddTorrent(self.mode), "unselected_connector");
                    icon = "󰄱";
                }
                Row::new(vec![Line::from(icon), Line::from(con.name.to_string())]).style(style)
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

    #[expect(unused)]
    fn split_area(&self, inner_area: Rect) -> (Rect, Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(2),
            ])
            .split(inner_area);
        (chunks[0], chunks[1], chunks[2])
    }
}
