use ratatui::{style::Style, text::Line};

use crate::{
    connectors::ConnectorName,
    domain::torrent::{Direction, State, TorrentInfo},
    settings::{Settings, styles::StyleMode},
    ui::{
        assets::{self, Symbols},
        components::torrent_list::table_layout::TableLayout,
    },
};

pub fn status_with_name(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    let icon = assets::Icons::status(info);
    Line::from(format!("{} {}", icon, info.name)).style(helpers::style(
        "status_with_name",
        &info.state,
        settings,
    ))
}

pub fn connector_with_folder(
    connector: ConnectorName,
    info: &TorrentInfo,
    settings: &Settings,
) -> Line<'static> {
    Line::from(format!("{}:{}", connector, info.output_folder)).style(helpers::style(
        "connector_with_folder",
        &info.state,
        settings,
    ))
}

pub fn peers_icon(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(assets::Icons::PEERS.to_string()).style(helpers::style(
        "peers",
        &info.state,
        settings,
    ))
}

pub fn peers(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(format!("{}/{}", info.peer_live, info.peer_seen)).style(helpers::style(
        "peers",
        &info.state,
        settings,
    ))
}

pub fn total_size(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(helpers::format::size(info.total_bytes)).style(helpers::style(
        "total_size",
        &info.state,
        settings,
    ))
}

pub fn time_remaining(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    let content = info
        .time_remaining_secs
        .map(helpers::format::time)
        .unwrap_or(" ─ ".into());
    Line::from(content).style(helpers::style("time_remaining", &info.state, settings))
}

pub fn uploading_icon(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(format!("{} ", assets::Icons::UPLOADING)).style(helpers::net_style(
        info,
        Direction::Upload,
        settings,
    ))
}

pub fn downloading_icon(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(format!("{} ", assets::Icons::DOWNLOADING)).style(helpers::net_style(
        info,
        Direction::Download,
        settings,
    ))
}

pub fn upload_speed(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(format!(" {:.2}MB/s ", info.upload_speed_mpbs)).style(helpers::net_style(
        info,
        Direction::Upload,
        settings,
    ))
}

pub fn download_speed(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(format!(" {:.2}MB/s ", info.download_speed_mpbs)).style(helpers::net_style(
        info,
        Direction::Download,
        settings,
    ))
}

pub fn uploaded_size(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(helpers::format::size(info.uploaded_bytes)).style(helpers::net_style(
        info,
        Direction::Upload,
        settings,
    ))
}

pub fn downloaded_size(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(helpers::format::size(info.downloaded_bytes)).style(helpers::net_style(
        info,
        Direction::Download,
        settings,
    ))
}

pub fn rate(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    let ratio = info.uploaded_bytes as f32 / info.total_bytes as f32;
    Line::from(format!("{:.1} ", ratio)).style(helpers::net_style(
        info,
        Direction::Upload,
        settings,
    ))
}

pub fn progress_percent(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    let ratio = info.downloaded_bytes as f32 / info.total_bytes as f32;
    let content = if ratio >= 1.0 {
        "100%".into()
    } else {
        format!("{:.1}%", ratio * 100.0)
    };
    Line::from(content).style(helpers::net_style(info, Direction::Download, settings))
}

pub fn divider(width: u16, info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    Line::from(Symbols::ROW_DIVIDER.repeat(width.into())).style(helpers::style(
        "dividers",
        &info.state,
        settings,
    ))
}

pub fn info_divider(info: &TorrentInfo, table_width: u16, settings: &Settings) -> Line<'static> {
    let len = table_width.saturating_sub(TableLayout::fixed_cols_total() + 2);
    divider(len, info, settings)
}

pub fn peers_divider(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    divider(TableLayout::PEERS_COLUMN_WIDTH, info, settings)
}

pub fn size_with_time_divider(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    divider(TableLayout::SIZE_WITH_TIME_COLUMN_WIDTH, info, settings)
}

pub fn ul_dl_icons_divider(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    divider(TableLayout::UL_DL_ICONS_COLUMN_WIDTH, info, settings)
}

pub fn speed_divider(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    divider(TableLayout::SPEED_COLUMN_WIDTH, info, settings)
}

pub fn progress_divider(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    divider(TableLayout::PROGRESS_COLUMN_WIDTH, info, settings)
}

pub fn rate_divider(info: &TorrentInfo, settings: &Settings) -> Line<'static> {
    divider(TableLayout::RATE_COLUMN_WIDTH, info, settings)
}

mod helpers {
    use super::*;
    pub(super) fn style(key: &str, state: &State, settings: &Settings) -> Style {
        let mode = StyleMode::from(state);
        settings.styles.get_style(&mode, key)
    }

    pub(super) fn net_style(
        info: &TorrentInfo,
        direction: Direction,
        settings: &Settings,
    ) -> Style {
        if info.speed(&direction) > 0.0 {
            settings
                .styles
                .get_style(&StyleMode::Active, direction.to_str())
        } else {
            style("default", &info.state, settings)
        }
    }
    pub(super) mod format {
        const KB: f32 = 1024.0;
        const MB: f32 = KB * 1024.0;
        const GB: f32 = MB * 1024.0;
        const TB: f32 = GB * 1024.0;

        const MIN: usize = 60;
        const HOUR: usize = MIN * 60;
        const DAY: usize = HOUR * 24;
        const WEEK: usize = DAY * 7;
        const MONTH: usize = DAY * 30;
        const YEAR: usize = DAY * 365;

        pub fn size(bytes: usize) -> String {
            let n = bytes as f32;
            if n < KB {
                format!(" {n}B ")
            } else if n < MB {
                format!(" {:.1}KB ", n / KB)
            } else if n < GB {
                format!(" {:.1}MB ", n / MB)
            } else if n < TB {
                format!(" {:.1}GB ", n / GB)
            } else {
                format!(" {:.1}TB ", n / TB)
            }
        }
        pub fn time(secs: usize) -> String {
            if secs < MIN {
                format!(" {secs}s ")
            } else if secs < HOUR {
                format!(" {}m {}s ", secs / MIN, secs % MIN)
            } else if secs < DAY {
                format!(" {}h {}m ", secs / HOUR, (secs % HOUR) / MIN)
            } else if secs < WEEK {
                format!(" {}d {}h ", secs / DAY, (secs % DAY) / HOUR)
            } else if secs < MONTH {
                format!(" {}w {}d ", secs / WEEK, (secs % WEEK) / DAY)
            } else if secs < YEAR {
                format!(" {}M {}d ", secs / MONTH, (secs % MONTH) / WEEK)
            } else {
                format!(" {}Y ", secs / YEAR)
            }
        }
    }
}
