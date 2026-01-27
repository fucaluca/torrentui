// TODO: Messy code. Needs refactoring, but no time now.
//       After release, figure out:
//       - Torrent selection logic
//       - State validation
//       - Optimistic UI updates
//       - Code duplication in pause_toggle
use std::collections::BTreeMap;
use std::sync::Arc;

use ratatui::layout::Alignment;
use ratatui::text::Line;
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
use crate::ui::torrent_list::table_layout::TableLayout;

mod cell;
mod column;
mod table_layout;

pub struct TorrentList {
    table_state: TableState,
    torrent_list: BTreeMap<ConnectorName, Vec<TorrentInfo>>,
    torrent_ids: Vec<(ConnectorName, usize)>,
}

impl Drawable for TorrentList {
    fn draw(&mut self, buf: &mut Buffer, area: Rect, settings: &Settings) {
        let rows = self.torrent_ids.iter().filter_map(|(connector_name, idx)| {
            self.torrent_list
                .get(connector_name)?
                .get(*idx)
                .map(|torrent_info| {
                    self.build_row(
                        Arc::clone(connector_name),
                        torrent_info,
                        settings,
                        area.width,
                    )
                })
        });
        let table = Table::new(rows, TableLayout::widths())
            .column_spacing(0)
            .block(self.create_block())
            .style(settings.styles.get_style(&StyleMode::Table, "default"))
            .row_highlight_style(settings.styles.get_style(&StyleMode::Table, "highlight"));
        StatefulWidget::render(table, area, buf, &mut self.table_state);
    }
}

impl TorrentList {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            torrent_list: BTreeMap::new(),
            torrent_ids: vec![],
        }
    }

    pub fn insert(&mut self, connector_name: ConnectorName, torrent_list: Vec<TorrentInfo>) {
        self.torrent_list.insert(connector_name, torrent_list);
    }

    pub fn update_table(&mut self) {
        self.torrent_ids = self
            .torrent_list
            .iter()
            .flat_map(|(connector_name, torrent_list)| {
                (0..torrent_list.len()).map(|i| (Arc::clone(connector_name), i))
            })
            .collect();
        /* let torrents_count = self.torrent_list.len();
        let mut rows: Vec<Row<'_>> = Vec::with_capacity(torrents_count);
        let mut torrent_ids: Vec<(ConnectorName, usize)> = Vec::with_capacity(torrents_count);
        for (connector_name, torrent_list) in &self.torrent_list {
            for (i, torrent_info) in torrent_list.iter().enumerate() {
                let row = self.build_row(Arc::clone(connector_name), torrent_info, settings);
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
                    .title_bottom(
                        Line::from(format!(" v{} ", env!("CARGO_PKG_VERSION"))).right_aligned(),
                    )
                    .border_type(BorderType::Rounded),
            )
            .style(settings.styles.get_style(&StyleMode::Table, "default"))
            .row_highlight_style(settings.styles.get_style(&StyleMode::Table, "highlight")); */
    }

    fn build_row(
        &self,
        connector_name: ConnectorName,
        torrent_info: &TorrentInfo,
        settings: &Settings,
        width: u16,
    ) -> Row<'static> {
        let cell = Cell::new(connector_name, torrent_info, width, settings);

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

    fn create_block(&self) -> Block<'static> {
        Block::new()
            .title_top(" Torrents ")
            .borders(Borders::all())
            .title_bottom(Line::from(format!(" v{} ", env!("CARGO_PKG_VERSION"))).right_aligned())
            .border_type(BorderType::Rounded)
    }

    pub fn action(&mut self, action: Action) -> Option<(ConnectorName, ConnectorCommands)> {
        match action {
            Action::Up => self.select_prev(),
            Action::Down => self.select_next(),
            Action::GotoTop => self.select_first(),
            Action::GotoBottom => self.select_last(),
            Action::Forget => self.forget(),
            Action::Delete => self.delete(),
            Action::Pause => self.pause(),
            Action::Start => self.start(),
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

    fn forget(&mut self) -> Option<(ConnectorName, ConnectorCommands)> {
        self.connector_command(ActionKind::Forget)
    }

    fn delete(&mut self) -> Option<(ConnectorName, ConnectorCommands)> {
        self.connector_command(ActionKind::Delete)
    }

    fn pause(&mut self) -> Option<(ConnectorName, ConnectorCommands)> {
        self.connector_command(ActionKind::Pause)
    }

    fn start(&mut self) -> Option<(ConnectorName, ConnectorCommands)> {
        self.connector_command(ActionKind::Start)
    }

    fn pause_toggle(&mut self) -> Option<(ConnectorName, ConnectorCommands)> {
        let cmd =
            self.selected_torrent()
                .map(|(connector_name, torrent_info)| match torrent_info.state {
                    State::Paused => {
                        torrent_info.state = State::Active;
                        Some((
                            ConnectorName::clone(connector_name),
                            ConnectorCommands::Action {
                                kind: ActionKind::Start,
                                info_hash: torrent_info.info_hash.clone(),
                            },
                        ))
                    }
                    _ => {
                        torrent_info.state = State::Paused;
                        Some((
                            ConnectorName::clone(connector_name),
                            ConnectorCommands::Action {
                                kind: ActionKind::Pause,
                                info_hash: torrent_info.info_hash.clone(),
                            },
                        ))
                    }
                })?;
        self.update_table();
        cmd
    }

    fn connector_command(
        &mut self,
        kind: ActionKind,
    ) -> Option<(ConnectorName, ConnectorCommands)> {
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

    fn selected_torrent(&mut self) -> Option<(&ConnectorName, &mut TorrentInfo)> {
        self.table_state
            .selected()
            .and_then(|selected| self.torrent_ids.get(selected))
            .and_then(|(connector_name, torrent_idx)| {
                self.torrent_list
                    .get_mut(connector_name)
                    .and_then(|torrent_list| torrent_list.get_mut(*torrent_idx))
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

    struct TestHelper {
        torrent_info: TorrentInfo,
        torrents: Vec<TorrentInfo>,
        component: TorrentList,
        connector_name: ConnectorName,
    }

    impl TestHelper {
        fn new() -> Self {
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
                component: TorrentList::new(),
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
                .insert(self.connector_name.clone(), self.torrents.clone());
            self.component.update_table();
            Ok(self)
        }

        fn draw(mut self, settings: &Settings) -> color_eyre::Result<Self> {
            let backend = TestBackend::new(82, 5);
            let mut terminal = Terminal::new(backend)?;

            terminal.draw(|frame| {
                let area = frame.area();
                let buffer = frame.buffer_mut();
                self.component.draw(buffer, area, settings);
            })?;
            Ok(self)
        }

        fn setup(self, settings: &Settings) -> color_eyre::Result<Self> {
            self.update_table()?.draw(settings)
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
        let helper = TestHelper::new().update_table()?;

        let width = 80;
        let height = 3;
        let row = helper.component.build_row(
            helper.connector_name.clone(),
            &helper.active(),
            &settings,
            width + 2,
        );
        let table = Table::new(vec![row], TableLayout::widths()).column_spacing(0);

        let backend = TestBackend::new(width, height);
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
        let mut helper = TestHelper::new().update_table()?;

        helper
            .component
            .insert(helper.connector_name.clone(), helper.torrents);
        helper.component.update_table();

        let backend = TestBackend::new(82, 5);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            helper.component.draw(buffer, area, &settings);
        })?;
        let buffer = terminal.backend().buffer();

        let expected_table: Vec<String> = [
            "╭ Torrents ──────────────────────────────────────────────────────────────────────╮",
            "│󰐊 Terminator.mp4                        891.1MB 󰕒   0.30MB/s   774.4MB     0.9 │",
            "│localhost:/home/user/video       12/34    5m 0s  󰇚  12.30MB/s   410.4MB    46.1%│",
            "│────────────────────────────────────────────────────────────────────────────────│",
            "╰──────────────────────────────────────────────────────────────────────── {vers} ╯",
        ]
        .iter()
        .map(|line| line.replace("{vers}", format!("v{}", env!("CARGO_PKG_VERSION")).as_str()))
        .collect::<Vec<String>>();

        let mut expected = Buffer::with_lines(expected_table);
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

        let mut helper = TestHelper::new();

        let torrents1 = vec![helper.active(), helper.paused(), helper.uploading()];
        let connector1 = ConnectorName::new("localhost".into());
        helper.component.insert(connector1, torrents1);
        helper.component.update_table();

        let torrents2 = vec![
            helper.downloading(),
            helper.initializing(),
            helper.errored(),
        ];
        let connector2 = ConnectorName::new("remote".into());
        helper.component.insert(connector2, torrents2);
        helper.component.update_table();

        let table_width = 82;
        let table_hight = 20;
        let backend = TestBackend::new(table_width, table_hight);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            helper.component.draw(buffer, area, &settings);
        })?;
        let buffer = terminal.backend().buffer();

        let expected_table = [
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
            "╰──────────────────────────────────────────────────────────────────────── {vers} ╯",
        ]
        .iter()
        .map(|line| line.replace("{vers}", format!("v{}", env!("CARGO_PKG_VERSION")).as_str()));

        let mut expected = Buffer::with_lines(expected_table);

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
        let mut helper = TestHelper::new();

        let torrents1 = vec![helper.active(), helper.paused(), helper.downloading()];
        let connector_name1 = ConnectorName::new("localhost".into());
        helper
            .component
            .insert(connector_name1.clone(), torrents1.clone());
        helper.component.update_table();

        let torrents2 = vec![helper.uploading(), helper.initializing(), helper.errored()];
        let connector_name2 = ConnectorName::new("remote".into());
        helper
            .component
            .insert(connector_name2.clone(), torrents2.clone());
        helper.component.update_table();

        let backend = TestBackend::new(82, 5);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            helper.component.draw(buffer, area, &settings);
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

        let mut helper = TestHelper::new().setup(&settings)?;
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
        let mut helper = TestHelper::new().setup(&settings)?;

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
        let mut helper = TestHelper::new().setup(&settings)?;

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
        let mut helper = TestHelper::new().setup(&settings)?;

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
        let mut helper = TestHelper::new().setup(&settings)?;

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
        let mut helper = TestHelper::new().setup(&settings)?;

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
