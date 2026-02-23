use ratatui::{style::Style, text::Line};

use crate::{
    connectors::ConnectorName,
    settings::{Settings, styles::StyleMode},
    torrent::{State, TorrentInfo},
    ui::{
        assets::{self, Symbols},
        components::torrent_list::table_layout::TableLayout,
    },
};

pub struct Cell {}

impl Cell {
    pub fn new() -> Self {
        Self {}
    }

    fn style(&self, key: &str, state: &State, settings: &Settings) -> Style {
        let mode = StyleMode::from(state);
        settings.styles.get_style(&mode, key)
    }

    fn net_style(&self, info: &TorrentInfo, direction: &str, settings: &Settings) -> Style {
        if info.speed(direction) > 0.0 {
            settings.styles.get_style(&StyleMode::Active, direction)
        } else {
            self.style("default", &info.state, settings)
        }
    }

    pub fn status_with_name(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        let icon = assets::Icons::status(info);
        Line::from(format!("{} {}", icon, info.name)).style(self.style(
            "status_with_name",
            &info.state,
            settings,
        ))
    }

    pub fn connector_with_folder(
        &self,
        connector: ConnectorName,
        info: &TorrentInfo,
        settings: &Settings,
    ) -> Line<'static> {
        Line::from(format!("{}:{}", connector, info.output_folder)).style(self.style(
            "connector_with_folder",
            &info.state,
            settings,
        ))
    }

    pub fn peers_icon(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(assets::Icons::PEERS.to_string()).style(self.style(
            "peers",
            &info.state,
            settings,
        ))
    }

    pub fn peers(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(format!("{}/{}", info.peer_live, info.peer_seen)).style(self.style(
            "peers",
            &info.state,
            settings,
        ))
    }

    pub fn total_size(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(Self::format_size(info.total_bytes)).style(self.style(
            "total_size",
            &info.state,
            settings,
        ))
    }

    pub fn time_remaining(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        let content = info
            .time_remaining_secs
            .map(Self::format_time)
            .unwrap_or(" ─ ".into());
        Line::from(content).style(self.style("time_remaining", &info.state, settings))
    }

    pub fn uploading_icon(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(format!("{} ", assets::Icons::UPLOADING))
            .style(self.net_style(info, "upload", settings))
    }

    pub fn downloading_icon(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(format!("{} ", assets::Icons::DOWNLOADING))
            .style(self.net_style(info, "download", settings))
    }

    pub fn upload_speed(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(format!(" {:.2}MB/s ", info.upload_speed_mpbs))
            .style(self.net_style(info, "upload", settings))
    }

    pub fn download_speed(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(format!(" {:.2}MB/s ", info.download_speed_mpbs))
            .style(self.net_style(info, "download", settings))
    }

    pub fn uploaded_size(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(Self::format_size(info.uploaded_bytes))
            .style(self.net_style(info, "upload", settings))
    }

    pub fn downloaded_size(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(Self::format_size(info.downloaded_bytes))
            .style(self.net_style(info, "download", settings))
    }

    pub fn rate(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        let ratio = info.uploaded_bytes as f32 / info.total_bytes as f32;
        Line::from(format!("{:.1} ", ratio)).style(self.net_style(info, "upload", settings))
    }

    pub fn progress_percent(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        let ratio = info.downloaded_bytes as f32 / info.total_bytes as f32;
        let content = if ratio >= 1.0 {
            "100%".into()
        } else {
            format!("{:.1}%", ratio * 100.0)
        };
        Line::from(content).style(self.net_style(info, "download", settings))
    }

    pub fn divider(&self, width: u16, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(width.into())).style(self.style(
            "dividers",
            &info.state,
            settings,
        ))
    }

    pub fn info_divider(
        &self,
        info: &TorrentInfo,
        table_width: u16,
        settings: &Settings,
    ) -> Line<'static> {
        let len = table_width.saturating_sub(TableLayout::fixed_cols_total() + 2);
        self.divider(len, info, settings)
    }

    pub fn peers_divider(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        self.divider(TableLayout::PEERS_COLUMN_WIDTH, info, settings)
    }

    pub fn size_with_time_divider(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        self.divider(TableLayout::SIZE_WITH_TIME_COLUMN_WIDTH, info, settings)
    }

    pub fn ul_dl_icons_divider(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        self.divider(TableLayout::UL_DL_ICONS_COLUMN_WIDTH, info, settings)
    }

    pub fn speed_divider(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        self.divider(TableLayout::SPEED_COLUMN_WIDTH, info, settings)
    }

    pub fn progress_divider(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        self.divider(TableLayout::PROGRESS_COLUMN_WIDTH, info, settings)
    }

    pub fn rate_divider(&self, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
        self.divider(TableLayout::RATE_COLUMN_WIDTH, info, settings)
    }
}

impl Cell {
    const KB: f32 = 1024.0;
    const MB: f32 = Self::KB * 1024.0;
    const GB: f32 = Self::MB * 1024.0;
    const TB: f32 = Self::GB * 1024.0;

    const MIN: usize = 60;
    const HOUR: usize = Self::MIN * 60;
    const DAY: usize = Self::HOUR * 24;
    const WEEK: usize = Self::DAY * 7;
    const MONTH: usize = Self::DAY * 30;
    const YEAR: usize = Self::DAY * 365;

    fn format_size(bytes: usize) -> String {
        let n = bytes as f32;
        if n < Self::KB {
            format!(" {n}B ")
        } else if n < Self::MB {
            format!(" {:.1}KB ", n / Self::KB)
        } else if n < Self::GB {
            format!(" {:.1}MB ", n / Self::MB)
        } else if n < Self::TB {
            format!(" {:.1}GB ", n / Self::GB)
        } else {
            format!(" {:.1}TB ", n / Self::TB)
        }
    }
    fn format_time(secs: usize) -> String {
        if secs < Self::MIN {
            format!(" {secs}s ")
        } else if secs < Self::HOUR {
            format!(" {}m {}s ", secs / Self::MIN, secs % Self::MIN)
        } else if secs < Self::DAY {
            format!(
                " {}h {}m ",
                secs / Self::HOUR,
                (secs % Self::HOUR) / Self::MIN
            )
        } else if secs < Self::WEEK {
            format!(
                " {}d {}h ",
                secs / Self::DAY,
                (secs % Self::DAY) / Self::HOUR
            )
        } else if secs < Self::MONTH {
            format!(
                " {}w {}d ",
                secs / Self::WEEK,
                (secs % Self::WEEK) / Self::DAY
            )
        } else if secs < Self::YEAR {
            format!(
                " {}M {}d ",
                secs / Self::MONTH,
                (secs % Self::MONTH) / Self::WEEK
            )
        } else {
            format!(" {}Y ", secs / Self::YEAR)
        }
    }
}
