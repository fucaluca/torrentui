use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Row, Table, Widget},
};

use crate::{
    action::Action,
    settings::{Settings, keybindings::KeyBindingsNode, styles::StyleMode},
    ui::Drawable,
};

const TITLE: &str = " Commands ";

#[derive(Clone)]
enum Desc {
    Text(String),
    Next(String),
}

pub struct WhichKey {
    keys: Vec<(String, Option<Desc>)>,
}

impl Drawable for WhichKey {
    fn draw(
        &mut self,
        buf: &mut ratatui::prelude::Buffer,
        area: ratatui::prelude::Rect,
        settings: &Settings,
    ) {
        if self.keys.is_empty() {
            return;
        }
        let key_style = settings.styles.get_style(&StyleMode::WhichKey, "key");
        let desc_style = settings.styles.get_style(&StyleMode::WhichKey, "desc");
        let next_style = settings.styles.get_style(&StyleMode::WhichKey, "next");
        let default_style = settings.styles.get_style(&StyleMode::WhichKey, "default");
        let rows = self
            .keys
            .iter()
            .filter_map(|(key, desc)| {
                /* let desc_span = match desc {
                    Some(txt) => Span::from(txt.clone()).style(desc_style),
                    None => Span::from("+").style(next_style),
                }; */
                let desc_span = desc.clone().map(|v| match v {
                    Desc::Text(txt) => Span::from(txt).style(desc_style),
                    Desc::Next(txt) => Span::from(txt).style(next_style),
                })?;

                Some(Row::new(vec![
                    Text::from(Span::from(key.clone()))
                        .style(key_style)
                        .alignment(Alignment::Right),
                    Text::from(desc_span),
                ]))
                /* if let Some(txt) = desc {
                    Row::new(vec![
                        Text::from(Span::from(key.clone()))
                            .style(key_style)
                            .alignment(Alignment::Right),
                        Text::from(desc_span),
                    ])
                } */
            })
            .collect::<Vec<Row>>();

        let height = self.keys.len() + 2;
        let (width_left, width_right) = self.calculate_max_char_widths();
        let width = width_left.max(2) + width_right.max(2) + 5;
        let widths = [
            Constraint::Min(width_left as u16),
            Constraint::Min(width_right as u16),
        ];
        let table = Table::new(rows, widths)
            .block(
                Block::default()
                    .title(TITLE)
                    .title_bottom(
                        Line::from(format!(" v{} ", env!("CARGO_PKG_VERSION"))).right_aligned(),
                    )
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::new(1, 1, 0, 0)),
            )
            .style(default_style);

        let bottom_area =
            self.bottom_right_rect(width.max(TITLE.len() + 2) as u16, height as u16, area);
        Clear.render(bottom_area, buf);
        table.render(bottom_area, buf);
    }
}

impl WhichKey {
    pub fn new() -> Self {
        Self { keys: vec![] }
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn update(&mut self, node: &KeyBindingsNode) {
        self.keys.clear();
        for (_, next_node) in &node.next {
            let description = match next_node.description.clone() {
                Some(desc) => Some(Desc::Text(desc)),
                None => match next_node.action {
                    // Action::Next => Some("+".into()),
                    Action::Next => Some(Desc::Next("+".into())),
                    Action::NoOp => None,
                    a => Some(Desc::Text(a.to_string())),
                },
            };
            self.keys.push((next_node.display.clone(), description));
        }
    }

    fn bottom_right_rect(&self, width: u16, height: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Min(0),
                Constraint::Length(height),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Min(0),
                Constraint::Length(width),
            ])
            .split(popup_layout[2])[2]
    }

    fn calculate_max_char_widths(&self) -> (usize, usize) {
        let max_first = self
            .keys
            .iter()
            .map(|(key, _)| key.chars().count())
            .max()
            .unwrap_or(0);

        let max_second = self
            .keys
            .iter()
            .filter_map(|(_, desc)| desc.as_ref())
            .map(|desc| match desc {
                Desc::Text(txt) | Desc::Next(txt) => txt.chars().count(),
            })
            .max()
            .unwrap_or(0);

        (max_first, max_second)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use indexmap::IndexMap;
    use ratatui::{
        Terminal,
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style},
    };

    use crate::{
        action::Action,
        settings::{ConfigSource, Settings, keybindings::KeyBindingsNode},
        ui::{Drawable, which_key::WhichKey},
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn styles() -> color_eyre::Result<()> {
        let config_toml = r#"
            [styles.WhichKey]
            key = "green on rgb:0,0,0"
            desc = "blue on rgb:0,0,0"
            next = "red on rgb:0,0,0"
            default = "yellow on rgb:0,0,0"
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;

        let mut which_key = WhichKey::new();

        let node = KeyBindingsNode {
            display: "q".into(),
            action: Action::Quit,
            description: Some("Exit".into()),
            next: IndexMap::new(),
        };

        let mut keybindings = KeyBindingsNode::default();

        keybindings
            .next
            .insert(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE), node);

        which_key.update(&keybindings);

        let expected_lines: Vec<String> = ["╭ Commands ╮", "│   q Exit │", "╰── {vers} ╯"]
            .iter()
            .map(|line| line.replace("{vers}", format!("v{}", env!("CARGO_PKG_VERSION")).as_str()))
            .collect::<Vec<String>>();
        let mut expected = Buffer::with_lines(expected_lines);

        let width = 12;
        let height = 3;
        expected.set_style(
            Rect::new(10, 1, 2, 1),
            Style::default().fg(Color::Yellow).bg(Color::Rgb(0, 0, 0)),
        );
        expected.set_style(
            Rect::new(6, 1, 4, 1),
            Style::default().fg(Color::Blue).bg(Color::Rgb(0, 0, 0)),
        );
        expected.set_style(
            Rect::new(5, 1, 1, 1),
            Style::default().fg(Color::Yellow).bg(Color::Rgb(0, 0, 0)),
        );
        expected.set_style(
            Rect::new(2, 1, 3, 1),
            Style::default().fg(Color::Green).bg(Color::Rgb(0, 0, 0)),
        );
        expected.set_style(
            Rect::new(0, 0, width, 1),
            Style::default().fg(Color::Yellow).bg(Color::Rgb(0, 0, 0)),
        );
        expected.set_style(
            Rect::new(1, 1, 1, 1),
            Style::default().fg(Color::Yellow).bg(Color::Rgb(0, 0, 0)),
        );
        expected.set_style(
            Rect::new(0, 2, width, 1),
            Style::default().fg(Color::Yellow).bg(Color::Rgb(0, 0, 0)),
        );
        expected.set_style(
            Rect::new(0, 0, 1, 3),
            Style::default().fg(Color::Yellow).bg(Color::Rgb(0, 0, 0)),
        );
        expected.set_style(
            Rect::new(11, 0, 1, 3),
            Style::default().fg(Color::Yellow).bg(Color::Rgb(0, 0, 0)),
        );

        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            which_key.draw(buffer, area, &settings);
        })?;
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer, &expected);
        Ok(())
    }

    #[test]
    fn draw_nested() -> color_eyre::Result<()> {
        let config_toml = r#""#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;

        let mut which_key = WhichKey::new();

        let leaf = KeyBindingsNode {
            display: "q".into(),
            action: Action::Quit,
            description: Some("Quit".into()),
            next: IndexMap::new(),
        };
        let mut node1 = KeyBindingsNode {
            display: "q".into(),
            action: Action::Next,
            description: None,
            next: IndexMap::new(),
        };
        node1
            .next
            .insert(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE), leaf);

        let mut keybindings = KeyBindingsNode::default();

        keybindings
            .next
            .insert(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE), node1);

        which_key.update(&keybindings);

        let expected_lines: Vec<String> = ["╭ Commands ╮", "│    q +   │", "╰── {vers} ╯"]
            .iter()
            .map(|line| line.replace("{vers}", format!("v{}", env!("CARGO_PKG_VERSION")).as_str()))
            .collect::<Vec<String>>();
        let expected = Buffer::with_lines(expected_lines);

        let width = 12;
        let height = 3;

        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            which_key.draw(buffer, area, &settings);
        })?;
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer, &expected);
        Ok(())
    }
}
