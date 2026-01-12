use std::sync::Arc;

use ratatui::text::Line;

use crate::{
    torrent::TorrentInfo,
    ui::{
        assets::{self, Symbols},
        components::torrent_list::table_layout::TableLayout,
    },
};

const KB_IN_BYTES: f32 = 1024.0;
const MB_IN_BYTES: f32 = KB_IN_BYTES * 1024.0;
const GB_IN_BYTES: f32 = MB_IN_BYTES * 1024.0;
const TB_IN_BYTES: f32 = GB_IN_BYTES * 1024.0;

const MINUTES_IN_SECS: usize = 60;
const HOURS_IN_SECS: usize = MINUTES_IN_SECS * 60;
const DAY_IN_SECS: usize = HOURS_IN_SECS * 24;
const WEEK_IN_SECS: usize = DAY_IN_SECS * 7;
const MONTH_IN_SECS: usize = DAY_IN_SECS * 30;
const YEAR_IN_SECS: usize = DAY_IN_SECS * 365;

pub struct CellHelper {
    connector_name: Arc<String>,
    torrent_info: TorrentInfo,
    table_width: u16,
}

impl CellHelper {
    pub fn new(connector_name: Arc<String>, torrent_info: TorrentInfo, table_width: u16) -> Self {
        Self {
            connector_name,
            torrent_info,
            table_width,
        }
    }

    pub fn status_with_name(&self) -> Line<'static> {
        let status_icon = assets::Icons::status(&self.torrent_info);
        let mut status_icon_with_name = String::new();
        status_icon_with_name.push(status_icon);
        status_icon_with_name.push(' ');
        status_icon_with_name.push_str(&self.torrent_info.name);
        Line::from(status_icon_with_name)
    }

    pub fn connector_with_folder(&self) -> Line<'static> {
        let mut connector_with_folder = String::new();
        connector_with_folder.push_str(&self.connector_name);
        connector_with_folder.push(':');
        connector_with_folder.push_str(&self.torrent_info.output_folder);
        Line::from(connector_with_folder)
    }

    pub fn peers(&self) -> Line<'static> {
        Line::from(format!(
            "{}/{}",
            self.torrent_info.peer_live, self.torrent_info.peer_seen
        ))
    }

    pub fn total_size(&self) -> Line<'static> {
        Line::from(size_from_bytes(self.torrent_info.total_bytes))
    }

    pub fn downloaded_size(&self) -> Line<'static> {
        Line::from(size_from_bytes(self.torrent_info.downloaded_bytes))
    }

    pub fn uploaded_size(&self) -> Line<'static> {
        Line::from(size_from_bytes(self.torrent_info.uploaded_bytes))
    }

    pub fn time_remaining(&self) -> Line<'static> {
        let time_remaining = if let Some(secs) = self.torrent_info.time_remaining_secs {
            time_from_secs(secs)
        } else {
            String::from(" ─ ")
        };
        Line::from(time_remaining)
    }

    pub fn upload_speed(&self) -> Line<'static> {
        Line::from(format!(" {:.2}MB/s ", self.torrent_info.upload_speed_mpbs))
    }

    pub fn download_speed(&self) -> Line<'static> {
        Line::from(format!(
            " {:.2}MB/s ",
            self.torrent_info.download_speed_mpbs
        ))
    }

    pub fn rate(&self) -> Line<'static> {
        Line::from(format!(
            "{:.1}",
            self.torrent_info.uploaded_bytes as f32 / self.torrent_info.total_bytes as f32
        ))
    }

    pub fn progress_percent(&self) -> Line<'static> {
        let progress_percent = if self.torrent_info.downloaded_bytes
            == self.torrent_info.total_bytes
        {
            "100%".into()
        } else {
            format!(
                "{:.1}%",
                (self.torrent_info.downloaded_bytes as f32 / self.torrent_info.total_bytes as f32)
                    * 100.0
            )
        };
        Line::from(progress_percent)
    }

    pub fn peers_icon(&self) -> Line<'static> {
        Line::from(assets::Icons::PEERS.to_string())
    }

    pub fn peers_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::PEERS_COLUMN_WIDTH.into()))
    }

    pub fn size_with_time_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::SIZE_WITH_TIME_COLUMN_WIDTH.into()))
    }

    pub fn uploading_icon(&self) -> Line<'static> {
        Line::from(format!("{} ", assets::Icons::UPLOADING))
    }

    pub fn downloading_icon(&self) -> Line<'static> {
        Line::from(format!("{} ", assets::Icons::DOWNLOADING))
    }

    pub fn ul_dl_icons_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::UL_DL_ICONS_COLUMN_WIDTH.into()))
    }

    pub fn speed_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::SPEED_COLUMN_WIDTH.into()))
    }

    pub fn progress_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::PROGRESS_COLUMN_WIDTH.into()))
    }

    pub fn rate_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::RATE_COLUMN_WIDTH.into()))
    }

    pub fn info_divider(&self) -> Line<'static> {
        let fixed_cols_total = TableLayout::fixed_cols_total();
        let divider_length = self.table_width.saturating_sub(fixed_cols_total) - 2;
        Line::from(Symbols::ROW_DIVIDER.repeat(divider_length.into()))
    }
}

fn size_from_bytes(bytes: usize) -> String {
    match bytes as f32 {
        n @ 0.0..KB_IN_BYTES => format!(" {n}B "),
        n @ KB_IN_BYTES..MB_IN_BYTES => {
            format!(" {:.1}KB ", n / KB_IN_BYTES)
        }
        n @ MB_IN_BYTES..GB_IN_BYTES => {
            format!(" {:.1}MB ", n / MB_IN_BYTES)
        }
        n @ GB_IN_BYTES..TB_IN_BYTES => {
            format!(" {:.1}GB ", n / GB_IN_BYTES)
        }
        n => format!(" {}TB ", n / TB_IN_BYTES),
    }
}

fn time_from_secs(secs: usize) -> String {
    match secs {
        n @ 0..MINUTES_IN_SECS => format!(" {n}s "),
        n @ MINUTES_IN_SECS..HOURS_IN_SECS => {
            format!(" {}m {}s ", n / MINUTES_IN_SECS, n % MINUTES_IN_SECS)
        }
        n @ HOURS_IN_SECS..DAY_IN_SECS => {
            format!(
                " {}h {}m ",
                n / HOURS_IN_SECS,
                n % HOURS_IN_SECS / MINUTES_IN_SECS
            )
        }
        n @ DAY_IN_SECS..WEEK_IN_SECS => {
            format!(
                " {}d {}h ",
                n / DAY_IN_SECS,
                n % DAY_IN_SECS / HOURS_IN_SECS
            )
        }
        n @ WEEK_IN_SECS..MONTH_IN_SECS => {
            format!(
                " {}w {}d ",
                n / WEEK_IN_SECS,
                n % WEEK_IN_SECS / DAY_IN_SECS
            )
        }
        n @ MONTH_IN_SECS..YEAR_IN_SECS => {
            format!(
                " {}M {}d ",
                n / MONTH_IN_SECS,
                n % MONTH_IN_SECS / WEEK_IN_SECS
            )
        }
        n => format!(" {}Y ", n / YEAR_IN_SECS),
    }
}
