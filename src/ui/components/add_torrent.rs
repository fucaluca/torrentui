use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, TableState, Widget},
};

use crate::{
    action::Action,
    connectors::{ConnectorCommands, ConnectorName},
    settings::Settings,
    ui::{Drawable, KeyEventResult, assets::Symbols},
};

pub struct AddTorrent<'a> {
    settings: &'a Settings,
    table: Table<'a>,
    table_state: TableState,
    connector_names: Vec<&'a ConnectorName>,
    display_text: String,
    cursor_position: usize,
    input_mode: bool,
}

impl Drawable for AddTorrent<'_> {
    fn draw(
        &mut self,
        buf: &mut ratatui::prelude::Buffer,
        area: ratatui::prelude::Rect,
        settings: &Settings,
    ) {
        self.update_table();
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
        self.table.clone().render(chunks[2], buf);

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

impl<'a> AddTorrent<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        let connector_names = settings.connectors.keys().collect::<Vec<&ConnectorName>>();
        Self {
            settings,
            table: Table::default(),
            table_state: TableState::default(),
            connector_names,
            display_text: String::new(),
            input_mode: false,
            cursor_position: 0,
        }
    }

    pub fn key_event(&mut self, key_event: KeyEvent) -> KeyEventResult {
        if !self.input_mode {
            return KeyEventResult::Ignored;
        }
        match key_event.code {
            KeyCode::Char(c) => self.display_text.push(c),
            KeyCode::Backspace => {
                self.display_text.pop();
            }
            KeyCode::Esc => self.disable_input(),
            _ => {}
        };
        KeyEventResult::Consumed
    }

    pub fn enable_input(&mut self) {
        self.input_mode = true;
    }

    pub fn disable_input(&mut self) {
        self.input_mode = false;
    }

    pub fn action(&mut self, action: Action) -> Option<(ConnectorName, ConnectorCommands)> {
        match action {
            Action::Backspace => {
                self.display_text.pop();
                None
            }
            _ => None,
        }
    }

    pub fn input(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char(ch) => self.display_text.push(ch),
            _ => {}
        }
    }

    pub fn update_table(&mut self) {
        let rows: Vec<Row<'_>> = self
            .settings
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
        self.table = Table::new(rows, widths).column_spacing(1);
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
