use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, TableState, Widget},
};
use snafu::{ResultExt, Snafu};
use tokio::sync::mpsc;

#[expect(unused)]
use crate::{
    action::Action,
    connectors::{ConnectorCommands, ConnectorName},
    settings::Settings,
    ui::{ActionResult, Drawable, assets::Symbols},
};
use crate::{
    action::{ActionError, GetActionFailedSnafu},
    mode::KeyMode,
    ui::assets,
};

#[derive(Debug, Snafu)]
pub enum AddTorrentError {
    #[snafu(display("Failed to initialize clipboard"))]
    ClipboardError { source: arboard::Error },
}

#[derive(Default)]
pub struct AddTorrent {
    #[expect(unused)]
    table_state: TableState,
    value: String,
    display_text: String,
    selection_start: Option<usize>,
    cursor_position: usize,
    input_mode: bool,
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
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

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

        if let Some(start) = self.selection_start {
            for i in start..self.cursor_position {
                if let Some(cell) = buf.cell_mut((chunks[0].x + i as u16, chunks[0].y)) {
                    cell.set_style(Style::default().fg(Color::Blue).bg(Color::White));
                }
            }
        }

        Paragraph::new(self.display_text.clone()).render(chunks[0], buf);
        Line::from(Symbols::ROW_DIVIDER.repeat(chunks[1].width as usize)).render(chunks[1], buf);
        let table = self.build_table(settings);
        table.render(chunks[2], buf);
    }
}

impl AddTorrent {
    pub fn new() -> Self {
        Self {
            display_text: String::from(assets::Symbols::CURSOR),
            ..Default::default()
        }
    }

    pub async fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        settings: &mut Settings,
        #[expect(unused)] connectors: &mut HashMap<String, mpsc::Sender<ConnectorCommands>>,
    ) -> Result<ActionResult, ActionError> {
        use Action::*;
        let maybe_action = settings
            .keybindings
            .action(KeyMode::AddTorrent, key_event)
            .context(GetActionFailedSnafu)?;
        let result = if self.input_mode {
            match maybe_action {
                Some(Escape) => {
                    self.input_mode = false;
                    if self.selection_start.is_some() {
                        self.selection_start = None;
                    }
                    Ok(ActionResult::Handled)
                }
                Some(Backspace) => {
                    if self.value.chars().count() > 0 && self.cursor_position > 0 {
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
                    self.input_mode = true;
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
                SelectAll => {
                    if self.selection_start.is_some() {
                        self.selection_start = Some(0);
                        self.cursor_position = self.value.chars().count();
                    } else {
                        self.selection_start = Some(0)
                    }
                    Ok(ActionResult::Handled)
                }
                _ => Ok(ActionResult::Unhandled(action)),
            }
        } else {
            Ok(ActionResult::Unhandled(Action::NoOp))
        };

        self.display_text = self.value.clone();
        self.display_text
            .insert(self.cursor_position, assets::Symbols::CURSOR);
        result
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

    fn build_table(&self, settings: &Settings) -> Table<'static> {
        let rows: Vec<Row<'_>> = settings
            .connectors
            .iter()
            .map(|(c, con)| {
                let checked = "󰄵";
                let unchecked = "󰄱";
                let icon = if *con.connector.selected() {
                    checked
                } else {
                    unchecked
                };
                Row::new(vec![Line::from(icon), Line::from(c.to_string())])
            })
            .collect();
        let widths = [Constraint::Length(2), Constraint::Fill(1)];
        Table::new(rows, widths).column_spacing(1)
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
