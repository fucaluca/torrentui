use std::collections::BTreeMap;
use std::sync::Arc;

use ratatui::layout::Alignment;
use ratatui::widgets::{Block, BorderType, Borders, Row, Table, TableState};
use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use crate::action::Action;
use crate::connectors::{ActionKind, ConnectorCommands, ConnectorName};
use crate::settings::Settings;
use crate::settings::styles::StyleMode;
use crate::torrent::{State, TorrentInfo};
use crate::ui::Drawable;
use crate::ui::components::torrent_list::cell::Cell;
use crate::ui::components::torrent_list::column::Column;
use crate::ui::torrent_list::style::StyleHelper;
use crate::ui::torrent_list::table_layout::TableLayout;

mod cell;
mod column;
mod style;
mod table_layout;

pub struct TorrentList<'a> {
    table: Table<'static>,
    table_state: TableState,
    table_width: u16,
    settings: &'a Settings,
    style_helper: StyleHelper<'a>,
    torrent_list: BTreeMap<ConnectorName, Vec<TorrentInfo>>,
    torrent_ids: Vec<(ConnectorName, usize)>,
}

impl Drawable for TorrentList<'_> {
    fn draw(&mut self, buf: &mut Buffer, area: Rect) {
        StatefulWidget::render(&self.table, area, buf, &mut self.table_state);
    }
}

impl<'a> TorrentList<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self {
            table: Table::default(),
            table_state: TableState::default(),
            table_width: 400,
            settings,
            style_helper: StyleHelper::new(&settings.styles),
            torrent_list: BTreeMap::new(),
            torrent_ids: vec![],
        }
    }

    pub fn update_table(&mut self, connector_name: ConnectorName, torrent_list: Vec<TorrentInfo>) {
        self.torrent_list.insert(connector_name, torrent_list);

        let mut rows: Vec<Row<'_>> = vec![];
        let mut torrent_ids: Vec<(ConnectorName, usize)> = vec![];
        for (connector_name, torrent_list) in &self.torrent_list {
            for (i, torrent_info) in torrent_list.iter().enumerate() {
                let row = self.build_row(Arc::clone(connector_name), torrent_info);
                rows.push(row);
                torrent_ids.push((Arc::clone(connector_name), i));
            }
        }
        self.torrent_ids = torrent_ids;

        self.table = Table::new(rows, TableLayout::widths())
            .column_spacing(0)
            .block(
                Block::new()
                    .title_top(" Torrents ")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            )
            .style(self.style_helper.get_style(&StyleMode::Table, "default"))
            .row_highlight_style(self.style_helper.get_style(&StyleMode::Table, "highlight"));
    }

    fn build_row(&self, connector_name: ConnectorName, torrent_info: &TorrentInfo) -> Row<'static> {
        let cell = Cell::new(
            connector_name,
            torrent_info,
            self.table_width,
            self.settings,
        );

        Row::new(vec![
            Column::builder()
                .top(cell.status_with_name())
                .bottom(cell.connector_with_folder())
                .divider(cell.info_divider())
                .alignment(Alignment::Left),
            Column::builder()
                .top(cell.peers_icon())
                .bottom(cell.peers())
                .divider(cell.peers_divider())
                .alignment(Alignment::Center),
            Column::builder()
                .top(cell.total_size())
                .bottom(cell.time_remaining())
                .divider(cell.size_with_time_divider())
                .alignment(Alignment::Center),
            Column::builder()
                .top(cell.uploading_icon())
                .bottom(cell.downloading_icon())
                .divider(cell.ul_dl_icons_divider())
                .alignment(Alignment::Center),
            Column::builder()
                .top(cell.upload_speed())
                .bottom(cell.download_speed())
                .divider(cell.speed_divider())
                .alignment(Alignment::Right),
            Column::builder()
                .top(cell.uploaded_size())
                .bottom(cell.downloaded_size())
                .divider(cell.progress_divider())
                .alignment(Alignment::Right),
            Column::builder()
                .top(cell.rate())
                .bottom(cell.progress_percent())
                .divider(cell.rate_divider())
                .alignment(Alignment::Right),
        ])
        .height(3)
    }

    pub fn action(&mut self, action: Action) -> Option<(ConnectorName, ConnectorCommands)> {
        match action {
            Action::Up => self.select_prev(),
            Action::Down => self.select_next(),
            Action::GotoTop => self.select_first(),
            Action::GotoBottom => self.select_last(),
            Action::Forget => self.connector_command(ActionKind::Forget),
            Action::Delete => self.connector_command(ActionKind::Delete),
            Action::Pause => self.connector_command(ActionKind::Pause),
            Action::Start => self.connector_command(ActionKind::Start),
            Action::PauseToggle => self.pause_toggle(),
            _ => None,
        }
    }

    fn select_prev(&mut self) -> Option<(ConnectorName, ConnectorCommands)> {
        self.table_state.select_previous();
        None
    }
    fn select_next(&mut self) -> Option<(ConnectorName, ConnectorCommands)> {
        self.table_state.select_next();
        None
    }
    fn select_first(&mut self) -> Option<(ConnectorName, ConnectorCommands)> {
        self.table_state.select_first();
        None
    }
    fn select_last(&mut self) -> Option<(ConnectorName, ConnectorCommands)> {
        self.table_state.select_last();
        None
    }

    fn pause_toggle(&self) -> Option<(ConnectorName, ConnectorCommands)> {
        self.selected_torrent()
            .map(|(connector_name, torrent_info)| match torrent_info.state {
                State::Paused => Some((
                    ConnectorName::clone(connector_name),
                    ConnectorCommands::Action {
                        kind: ActionKind::Start,
                        info_hash: torrent_info.info_hash.clone(),
                    },
                )),
                _ => Some((
                    ConnectorName::clone(connector_name),
                    ConnectorCommands::Action {
                        kind: ActionKind::Pause,
                        info_hash: torrent_info.info_hash.clone(),
                    },
                )),
            })?
    }

    fn connector_command(&self, kind: ActionKind) -> Option<(ConnectorName, ConnectorCommands)> {
        self.selected_torrent()
            .map(|(connector_name, torrent_info)| {
                (
                    ConnectorName::clone(connector_name),
                    ConnectorCommands::Action {
                        kind,
                        info_hash: torrent_info.info_hash.clone(),
                    },
                )
            })
    }

    fn selected_torrent(&self) -> Option<(&ConnectorName, &TorrentInfo)> {
        self.table_state
            .selected()
            .and_then(|selected| self.torrent_ids.get(selected))
            .and_then(|(connector_name, torrent_idx)| {
                self.torrent_list
                    .get(connector_name)
                    .and_then(|torrent_list| torrent_list.get(*torrent_idx))
                    .map(|torrent_info| (connector_name, torrent_info))
            })
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use assert_matches::assert_matches;
    use color_eyre::eyre::OptionExt;
    use fake::{Fake, Faker};
    use ratatui::{
        Terminal,
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style},
        widgets::Table,
    };

    use crate::{
        connectors::{ActionKind, ConnectorCommands, ConnectorName},
        settings::{ConfigSource, Settings},
        torrent::{InfoHash, State, TorrentInfo},
        ui::{Drawable, TorrentList, torrent_list::table_layout::TableLayout},
    };

    use pretty_assertions::assert_eq;

    struct TestHelper<'a> {
        torrent_info: TorrentInfo,
        torrents: Vec<TorrentInfo>,
        component: TorrentList<'a>,
        connector_name: ConnectorName,
    }

    impl<'a> TestHelper<'a> {
        fn new(settings: &'a Settings) -> Self {
            Self {
                torrent_info: TorrentInfo {
                    name: "Terminator.mp4".into(),
                    info_hash: Faker.fake::<InfoHash>(),
                    output_folder: "/home/user/video".into(),
                    finished: false,
                    state: State::Active,
                    downloaded_bytes: 430340192,
                    uploaded_bytes: 812020390,
                    total_bytes: 934403901,
                    download_speed_mpbs: 12.3,
                    upload_speed_mpbs: 0.3,
                    time_remaining_secs: Some(300),
                    peer_live: 12,
                    peer_seen: 34,
                },
                component: TorrentList::new(settings),
                connector_name: ConnectorName::new("localhost".into()),
                torrents: vec![],
            }
        }

        fn update_table(mut self) -> color_eyre::Result<Self> {
            self.torrents = vec![
                self.active(),
                self.paused(),
                self.uploading(),
                self.downloading(),
                self.initializing(),
                self.errored(),
            ];
            self.component
                .update_table(self.connector_name.clone(), self.torrents.clone());
            Ok(self)
        }

        fn draw(mut self) -> color_eyre::Result<Self> {
            let backend = TestBackend::new(82, 5);
            let mut terminal = Terminal::new(backend)?;

            terminal.draw(|frame| {
                let area = frame.area();
                let buffer = frame.buffer_mut();
                self.component.draw(buffer, area);
            })?;
            Ok(self)
        }

        fn setup(self) -> color_eyre::Result<Self> {
            self.update_table()?.draw()
        }

        fn active(&self) -> TorrentInfo {
            self.torrent_info.clone()
        }

        fn paused(&self) -> TorrentInfo {
            TorrentInfo {
                state: State::Paused,
                name: "Paused torrent.test".into(),
                upload_speed_mpbs: 0.0,
                download_speed_mpbs: 0.0,
                peer_live: 0,
                info_hash: Faker.fake::<InfoHash>(),
                ..self.torrent_info.clone()
            }
        }

        fn downloading(&self) -> TorrentInfo {
            TorrentInfo {
                state: State::Active,
                name: "Downloading torrent.test".into(),
                upload_speed_mpbs: 0.0,
                download_speed_mpbs: 9.2,
                peer_live: 33,
                info_hash: Faker.fake::<InfoHash>(),
                ..self.torrent_info.clone()
            }
        }

        fn uploading(&self) -> TorrentInfo {
            TorrentInfo {
                state: State::Active,
                name: "Uploading torrent.test".into(),
                upload_speed_mpbs: 4.3,
                download_speed_mpbs: 0.0,
                peer_live: 11,
                info_hash: Faker.fake::<InfoHash>(),
                ..self.torrent_info.clone()
            }
        }

        fn initializing(&self) -> TorrentInfo {
            TorrentInfo {
                state: State::Initializing,
                name: "Initializing torrent.test".into(),
                upload_speed_mpbs: 0.0,
                download_speed_mpbs: 0.0,
                peer_live: 0,
                downloaded_bytes: 4402104,
                time_remaining_secs: Some(12),
                info_hash: Faker.fake::<InfoHash>(),
                ..self.torrent_info.clone()
            }
        }

        fn errored(&self) -> TorrentInfo {
            TorrentInfo {
                state: State::Error,
                name: "Errored torrent.test".into(),
                upload_speed_mpbs: 0.0,
                download_speed_mpbs: 0.0,
                uploaded_bytes: 0,
                downloaded_bytes: 0,
                peer_live: 0,
                peer_seen: 0,
                time_remaining_secs: None,
                info_hash: Faker.fake::<InfoHash>(),
                ..self.torrent_info.clone()
            }
        }
    }

    #[test]
    fn draw_row() -> color_eyre::Result<()> {
        let config_toml = r#""#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let helper = TestHelper::new(&settings).update_table()?;

        let row = helper
            .component
            .build_row(helper.connector_name.clone(), &helper.active());
        let table = Table::new(vec![row], TableLayout::widths()).column_spacing(0);

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|frame| {
            frame.render_widget(table, frame.area());
        })?;

        let expected = Buffer::with_lines(vec![
            "󰐊 Terminator.mp4                        891.1MB 󰕒   0.30MB/s   774.4MB     0.9 ",
            "localhost:/home/user/video       12/34    5m 0s  󰇚  12.30MB/s   410.4MB    46.1%",
            "────────────────────────────────────────────────────────────────────────────────",
        ]);

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer, &expected);
        Ok(())
    }

    #[test]
    fn table_default_style() -> color_eyre::Result<()> {
        let config_toml = r#"
            [styles.Table]
            default = "red on blue"
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let mut helper = TestHelper::new(&settings).update_table()?;

        helper
            .component
            .update_table(helper.connector_name.clone(), helper.torrents);

        let backend = TestBackend::new(82, 5);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            helper.component.draw(buffer, area);
        })?;
        let buffer = terminal.backend().buffer();

        let mut expected = Buffer::with_lines(vec![
            "╭ Torrents ──────────────────────────────────────────────────────────────────────╮",
            "│󰐊 Terminator.mp4                        891.1MB 󰕒   0.30MB/s   774.4MB     0.9 │",
            "│localhost:/home/user/video       12/34    5m 0s  󰇚  12.30MB/s   410.4MB    46.1%│",
            "│────────────────────────────────────────────────────────────────────────────────│",
            "╰────────────────────────────────────────────────────────────────────────────────╯",
        ]);
        expected.set_style(
            Rect::new(0, 0, 82, 5),
            Style::default().fg(Color::Red).bg(Color::Blue),
        );
        assert_eq!(buffer, &expected);
        Ok(())
    }

    #[test]
    fn draw_table() -> color_eyre::Result<()> {
        let config_toml = r#"
            [styles.Active]
            upload = "black on yellow"
            download = "black on blue"

            [styles.Paused]
            default = "green on red"

            [styles.Initializing]
            default = "yellow on rgb:0,0,0"

            [styles.Error]
            default = "red on white"
        "#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;

        let mut helper = TestHelper::new(&settings);

        let torrents1 = vec![helper.active(), helper.paused(), helper.uploading()];
        let connector1 = ConnectorName::new("localhost".into());
        helper.component.update_table(connector1, torrents1);

        let torrents2 = vec![
            helper.downloading(),
            helper.initializing(),
            helper.errored(),
        ];
        let connector2 = ConnectorName::new("remote".into());
        helper.component.update_table(connector2, torrents2);

        let table_width = 82;
        let table_hight = 20;
        let backend = TestBackend::new(table_width, table_hight);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            helper.component.draw(buffer, area);
        })?;
        let buffer = terminal.backend().buffer();

        let mut expected = Buffer::with_lines(vec![
            "╭ Torrents ──────────────────────────────────────────────────────────────────────╮",
            "│󰐊 Terminator.mp4                        891.1MB 󰕒   0.30MB/s   774.4MB     0.9 │",
            "│localhost:/home/user/video       12/34    5m 0s  󰇚  12.30MB/s   410.4MB    46.1%│",
            "│────────────────────────────────────────────────────────────────────────────────│",
            "│󰏤 Paused torrent.test                   891.1MB 󰕒   0.00MB/s   774.4MB     0.9 │",
            "│localhost:/home/user/video       0/34     5m 0s  󰇚   0.00MB/s   410.4MB    46.1%│",
            "│────────────────────────────────────────────────────────────────────────────────│",
            "│󰐊 Uploading torrent.test                891.1MB 󰕒   4.30MB/s   774.4MB     0.9 │",
            "│localhost:/home/user/video       11/34    5m 0s  󰇚   0.00MB/s   410.4MB    46.1%│",
            "│────────────────────────────────────────────────────────────────────────────────│",
            "│󰐊 Downloading torrent.test              891.1MB 󰕒   0.00MB/s   774.4MB     0.9 │",
            "│remote:/home/user/video          33/34    5m 0s  󰇚   9.20MB/s   410.4MB    46.1%│",
            "│────────────────────────────────────────────────────────────────────────────────│",
            "│ Initializing torrent.test             891.1MB 󰕒   0.00MB/s   774.4MB     0.9 │",
            "│remote:/home/user/video          0/34      12s   󰇚   0.00MB/s     4.2MB     0.5%│",
            "│────────────────────────────────────────────────────────────────────────────────│",
            "│ Errored torrent.test                  891.1MB 󰕒   0.00MB/s        0B     0.0 │",
            "│remote:/home/user/video           0/0       ─    󰇚   0.00MB/s        0B     0.0%│",
            "│────────────────────────────────────────────────────────────────────────────────│",
            "╰────────────────────────────────────────────────────────────────────────────────╯",
        ]);

        expected.set_style(
            Rect::new(50, 1, 31, 1), // NOTE: uploading area torrent 1
            Style::default().fg(Color::Black).bg(Color::Yellow),
        );
        expected.set_style(
            Rect::new(50, 2, 31, 1), // NOTE: downloading area torrent 1
            Style::default().fg(Color::Black).bg(Color::Blue),
        );
        expected.set_style(
            Rect::new(1, 4, table_width - 2, 3), // NOTE: torrent 2 (paused)
            Style::default().fg(Color::Green).bg(Color::Red),
        );
        expected.set_style(
            Rect::new(50, 7, 31, 1), // NOTE: uploading area torrent 3
            Style::default().fg(Color::Black).bg(Color::Yellow),
        );
        expected.set_style(
            Rect::new(50, 11, 31, 1), // NOTE: downloading area torrent 4
            Style::default().fg(Color::Black).bg(Color::Blue),
        );
        expected.set_style(
            Rect::new(1, 13, table_width - 2, 3), // NOTE: torrent 5 (initializing)
            Style::default().fg(Color::Yellow).bg(Color::Rgb(0, 0, 0)),
        );
        expected.set_style(
            Rect::new(1, 16, table_width - 2, 3), // NOTE: torrent 6 (error)
            Style::default().fg(Color::Red).bg(Color::White),
        );

        assert_eq!(buffer, &expected);

        Ok(())
    }

    #[test]
    fn selected_torrent() -> color_eyre::Result<()> {
        let config_toml = r#""#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let mut helper = TestHelper::new(&settings);

        let torrents1 = vec![helper.active(), helper.paused(), helper.downloading()];
        let connector_name1 = ConnectorName::new("localhost".into());
        helper
            .component
            .update_table(connector_name1.clone(), torrents1.clone());

        let torrents2 = vec![helper.uploading(), helper.initializing(), helper.errored()];
        let connector_name2 = ConnectorName::new("remote".into());
        helper
            .component
            .update_table(connector_name2.clone(), torrents2.clone());

        let backend = TestBackend::new(82, 5);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            helper.component.draw(buffer, area);
        })?;

        let mut all_torrents = torrents1.clone();
        all_torrents.extend(torrents2);

        helper.component.table_state.select(Some(0));
        let (selected_connector_name, selected_torrent) =
            helper.component.selected_torrent().unwrap();

        assert_eq!(selected_connector_name, &connector_name1);
        assert_eq!(selected_torrent.info_hash, all_torrents[0].info_hash);

        helper.component.table_state.select(Some(1));
        let (selected_connector_name, selected_torrent) =
            helper.component.selected_torrent().unwrap();

        assert_eq!(selected_connector_name, &connector_name1);
        assert_eq!(selected_torrent.info_hash, all_torrents[1].info_hash);

        helper.component.table_state.select(Some(2));
        let (selected_connector_name, selected_torrent) =
            helper.component.selected_torrent().unwrap();

        assert_eq!(selected_connector_name, &connector_name1);
        assert_eq!(selected_torrent.info_hash, all_torrents[2].info_hash);

        helper.component.table_state.select(Some(3));
        let (selected_connector_name, selected_torrent) =
            helper.component.selected_torrent().unwrap();

        assert_eq!(selected_connector_name, &connector_name2);
        assert_eq!(selected_torrent.info_hash, all_torrents[3].info_hash);

        helper.component.table_state.select(Some(4));
        let (selected_connector_name, selected_torrent) =
            helper.component.selected_torrent().unwrap();

        assert_eq!(selected_connector_name, &connector_name2);
        assert_eq!(selected_torrent.info_hash, all_torrents[4].info_hash);

        helper.component.table_state.select(Some(5));
        let (selected_connector_name, selected_torrent) =
            helper.component.selected_torrent().unwrap();

        assert_eq!(selected_connector_name, &connector_name2);
        assert_eq!(selected_torrent.info_hash, all_torrents[5].info_hash);
        Ok(())
    }

    #[test]
    fn connector_command() -> color_eyre::Result<()> {
        let config_toml = r#""#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;

        let mut helper = TestHelper::new(&settings).setup()?;
        let action_kind = ActionKind::Forget;

        helper.component.table_state.select(Some(1));
        let (connector_name, command) = helper
            .component
            .connector_command(action_kind.clone())
            .unwrap();

        assert_eq!(Arc::ptr_eq(&connector_name, &helper.connector_name), true);
        assert_matches!(command, ConnectorCommands::Action { kind, info_hash } => {
            assert_eq!(kind, action_kind);
            assert_eq!(info_hash, helper.torrents[1].info_hash);
        });
        Ok(())
    }

    #[test]
    fn pause_toggle() -> color_eyre::Result<()> {
        let config_toml = r#""#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let mut helper = TestHelper::new(&settings).setup()?;

        helper.component.table_state.select(Some(1));

        let (connector, command) = helper.component.pause_toggle().unwrap();

        assert_eq!(connector, helper.connector_name);
        assert_matches!(
            command,
            ConnectorCommands::Action { kind, info_hash } => {
                assert_eq!(kind, ActionKind::Start);
                assert_eq!(info_hash, helper.torrents[1].info_hash);
            }
        );

        helper.component.table_state.select(Some(0));
        let (connector, command) = helper.component.pause_toggle().unwrap();

        assert_eq!(connector, helper.connector_name);
        assert_matches!(
            command,
            ConnectorCommands::Action { kind, info_hash } => {
                assert_eq!(kind, ActionKind::Pause);
                assert_eq!(info_hash, helper.torrents[0].info_hash);
            }
        );

        Ok(())
    }

    #[test]
    fn select_first() -> color_eyre::Result<()> {
        let config_toml = r#""#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let mut helper = TestHelper::new(&settings).setup()?;

        let result = helper.component.select_first();
        assert_eq!(result.is_none(), true);

        let selected = helper
            .component
            .table_state
            .selected()
            .ok_or_eyre("No row selected")?;

        let (connector, torrent_idx) = helper
            .component
            .torrent_ids
            .get(selected)
            .ok_or_eyre("Selected torrent_idx not found")?;

        let selected_torrent = helper
            .component
            .torrent_list
            .get(connector)
            .ok_or_eyre(r#"Connector "{connector}" not found in torrent_list"#)?
            .get(*torrent_idx)
            .ok_or_eyre(r#"torrent_idx "{torrent_idx}" not found in torrent_list"#)?;

        let expected_torrent = helper
            .torrents
            .first()
            .ok_or_eyre("First torrent_info not found")?;
        assert_eq!(selected_torrent.info_hash, expected_torrent.info_hash);
        Ok(())
    }

    #[test]
    fn select_next() -> color_eyre::Result<()> {
        let config_toml = r#""#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let mut helper = TestHelper::new(&settings).setup()?;

        let result = helper.component.select_first();
        assert_eq!(result.is_none(), true);

        let result = helper.component.select_next();
        assert_eq!(result.is_none(), true);

        let selected = helper
            .component
            .table_state
            .selected()
            .ok_or_eyre("No row selected")?;

        let (connector, torrent_idx) = helper
            .component
            .torrent_ids
            .get(selected)
            .ok_or_eyre("Selected torrent_idx not found")?;

        let selected_torrent = helper
            .component
            .torrent_list
            .get(connector)
            .ok_or_eyre(r#"Connector "{connector}" not found in torrent_list"#)?
            .get(*torrent_idx)
            .ok_or_eyre(r#"torrent_idx "{torrent_idx}" not found in torrent_list"#)?;

        let expected_torrent = helper
            .torrents
            .get(1)
            .ok_or_eyre("Next torrent_info not found")?;
        assert_eq!(selected_torrent.info_hash, expected_torrent.info_hash);
        Ok(())
    }

    #[test]
    fn select_prev() -> color_eyre::Result<()> {
        let config_toml = r#""#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let mut helper = TestHelper::new(&settings).setup()?;

        helper.component.table_state.select(Some(3));

        let result = helper.component.select_prev();
        assert_eq!(result.is_none(), true);

        let selected = helper
            .component
            .table_state
            .selected()
            .ok_or_eyre("No row selected")?;

        let (connector, torrent_idx) = helper
            .component
            .torrent_ids
            .get(selected)
            .ok_or_eyre("Selected torrent_idx not found")?;

        let selected_torrent = helper
            .component
            .torrent_list
            .get(connector)
            .ok_or_eyre(r#"Connector "{connector}" not found in torrent_list"#)?
            .get(*torrent_idx)
            .ok_or_eyre(r#"torrent_idx "{torrent_idx}" not found in torrent_list"#)?;

        let expected_torrent = helper
            .torrents
            .get(2)
            .ok_or_eyre("Next torrent_info not found")?;
        assert_eq!(selected_torrent.info_hash, expected_torrent.info_hash);
        Ok(())
    }

    // TODO: Note: until the table is rendered, the number of rows
    // is not known, so the index is set to `usize::MAX` and will
    // be corrected when the table is rendered
    #[ignore]
    #[test]
    fn select_last() -> color_eyre::Result<()> {
        let config_toml = r#""#;
        let config_source = ConfigSource::String(config_toml.into());
        let settings = Settings::new(config_source)?;
        let mut helper = TestHelper::new(&settings).setup()?;

        std::thread::sleep(Duration::from_secs(2));
        let result = helper.component.select_last();
        assert_eq!(result.is_none(), true);

        let selected = helper
            .component
            .table_state
            .selected()
            .ok_or_eyre("No row selected")?;

        let (connector, torrent_idx) = helper
            .component
            .torrent_ids
            .get(selected)
            .ok_or_eyre("Selected torrent_id not found")?;

        let selected_torrent = helper
            .component
            .torrent_list
            .get(connector)
            .ok_or_eyre(r#"Connector "{connector}" not found in torrent_list"#)?
            .get(*torrent_idx)
            .ok_or_eyre(r#"torrent_idx "{torrent_idx}" not found in torrent_list"#)?;

        let expected_torrent = helper
            .torrents
            .first()
            .ok_or_eyre("First torrent_info not found")?;
        assert_eq!(selected_torrent.info_hash, expected_torrent.info_hash);
        Ok(())
    }

    #[ignore]
    #[test]
    fn action() {}
}
