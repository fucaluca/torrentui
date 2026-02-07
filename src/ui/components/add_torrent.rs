use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, TableState, Widget},
};
use snafu::ResultExt;
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
};

pub struct AddTorrent {
    #[expect(unused)]
    table_state: TableState,
    display_text: String,
    #[expect(unused)]
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

        Paragraph::new(self.display_text.clone()).render(chunks[0], buf);
        Line::from(Symbols::ROW_DIVIDER.repeat(chunks[1].width as usize)).render(chunks[1], buf);
        // self.table.clone().render(chunks[2], buf);
        let table = self.build_table(settings);
        table.render(chunks[2], buf);

        /* Widget::render(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
            centered_area,
            buf,
        ); */
        // Widget::render(
        //     Paragraph::new(Text::from(vec![Line::from("asdf"), Line::from("iii")])).block(
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .border_type(ratatui::widgets::BorderType::Rounded),
        //     ),
        //     centered_area,
        //     buf,
        // );
    }
}

impl AddTorrent {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            display_text: String::new(),
            input_mode: false,
            cursor_position: 0,
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
        if self.input_mode {
            match maybe_action {
                Some(Escape) => {
                    self.input_mode = false;
                    Ok(ActionResult::Handled)
                }
                Some(Backspace) => {
                    self.display_text.pop();
                    Ok(ActionResult::Handled)
                }
                Some(_) | None => {
                    self.input(key_event.code);
                    Ok(ActionResult::Handled)
                }
            }
        } else if let Some(action) = maybe_action {
            match action {
                Input => {
                    self.input_mode = true;
                    Ok(ActionResult::Handled)
                }
                _ => Ok(ActionResult::Unhandled(action)),
            }
        } else {
            Ok(ActionResult::Unhandled(Action::NoOp))
        }
    }

    pub fn input(&mut self, key_code: KeyCode) {
        #[expect(clippy::single_match)]
        match key_code {
            KeyCode::Char(ch) => self.display_text.push(ch),
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
