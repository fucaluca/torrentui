use std::sync::Arc;

use ratatui::{style::Style, text::Line};

use crate::{
    settings::{Settings, styles::StyleMode},
    torrent::TorrentInfo,
    ui::{
        assets::{self, Symbols},
        components::torrent_list::table_layout::TableLayout,
    },
};

pub struct Cell<'a> {
    connector_name: Arc<String>,
    torrent_info: &'a TorrentInfo,
    table_width: u16,
    settings: &'a Settings,
}

impl<'a> Cell<'a> {
    pub fn new(
        connector_name: Arc<String>,
        torrent_info: &'a TorrentInfo,
        table_width: u16,
        settings: &'a Settings,
    ) -> Self {
        Self {
            connector_name,
            torrent_info,
            table_width,
            settings,
        }
    }

    fn get_upload_style(&self, mode: &StyleMode) -> Style {
        if self.torrent_info.upload_speed_mpbs > 0.0 {
            self.settings.styles.get_style(&StyleMode::Active, "upload")
        } else {
            self.settings.styles.get_style(mode, "default")
        }
    }

    fn get_download_style(&self, mode: &StyleMode) -> Style {
        if self.torrent_info.download_speed_mpbs > 0.0 {
            self.settings
                .styles
                .get_style(&StyleMode::Active, "download")
        } else {
            self.settings.styles.get_style(mode, "default")
        }
    }

    pub fn status_with_name(&self) -> Line<'static> {
        let status_icon = assets::Icons::status(&self.torrent_info);
        let mut status_icon_with_name = String::new();
        status_icon_with_name.push(status_icon);
        status_icon_with_name.push(' ');
        status_icon_with_name.push_str(&self.torrent_info.name);
        Line::from(status_icon_with_name).style(self.settings.styles.get_style(
            &StyleMode::from(&self.torrent_info.state),
            "status_with_name",
        ))
    }

    pub fn connector_with_folder(&self) -> Line<'static> {
        let mut connector_with_folder = String::new();
        connector_with_folder.push_str(&self.connector_name);
        connector_with_folder.push(':');
        connector_with_folder.push_str(&self.torrent_info.output_folder);
        Line::from(connector_with_folder).style(self.settings.styles.get_style(
            &StyleMode::from(&self.torrent_info.state),
            "connector_with_folder",
        ))
    }

    pub fn peers_icon(&self) -> Line<'static> {
        Line::from(assets::Icons::PEERS.to_string()).style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "peers"),
        )
    }

    pub fn peers(&self) -> Line<'static> {
        Line::from(format!(
            "{}/{}",
            self.torrent_info.peer_live, self.torrent_info.peer_seen
        ))
        .style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "peers"),
        )
    }

    pub fn total_size(&self) -> Line<'static> {
        Line::from(self.size_from_bytes(self.torrent_info.total_bytes)).style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "total_size"),
        )
    }

    pub fn time_remaining(&self) -> Line<'static> {
        let time_remaining = if let Some(secs) = self.torrent_info.time_remaining_secs {
            self.time_from_secs(secs)
        } else {
            String::from(" ─ ")
        };
        Line::from(time_remaining).style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "time_remaining"),
        )
    }

    pub fn uploading_icon(&self) -> Line<'static> {
        Line::from(format!("{} ", assets::Icons::UPLOADING))
            .style(self.get_upload_style(&StyleMode::from(&self.torrent_info.state)))
    }

    pub fn downloading_icon(&self) -> Line<'static> {
        Line::from(format!("{} ", assets::Icons::DOWNLOADING))
            .style(self.get_download_style(&StyleMode::from(&self.torrent_info.state)))
    }

    pub fn upload_speed(&self) -> Line<'static> {
        Line::from(format!(" {:.2}MB/s ", self.torrent_info.upload_speed_mpbs))
            .style(self.get_upload_style(&StyleMode::from(&self.torrent_info.state)))
    }

    pub fn download_speed(&self) -> Line<'static> {
        Line::from(format!(
            " {:.2}MB/s ",
            self.torrent_info.download_speed_mpbs
        ))
        .style(self.get_download_style(&StyleMode::from(&self.torrent_info.state)))
    }

    pub fn uploaded_size(&self) -> Line<'static> {
        Line::from(self.size_from_bytes(self.torrent_info.uploaded_bytes))
            .style(self.get_upload_style(&StyleMode::from(&self.torrent_info.state)))
    }

    pub fn downloaded_size(&self) -> Line<'static> {
        Line::from(self.size_from_bytes(self.torrent_info.downloaded_bytes))
            .style(self.get_download_style(&StyleMode::from(&self.torrent_info.state)))
    }

    pub fn rate(&self) -> Line<'static> {
        Line::from(format!(
            "{:.1} ",
            self.torrent_info.uploaded_bytes as f32 / self.torrent_info.total_bytes as f32
        ))
        .style(self.get_upload_style(&StyleMode::from(&self.torrent_info.state)))
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
            .style(self.get_download_style(&StyleMode::from(&self.torrent_info.state)))
    }

    pub fn info_divider(&self) -> Line<'static> {
        let fixed_cols_total = TableLayout::fixed_cols_total();
        let divider_length = self.table_width.saturating_sub(fixed_cols_total) - 2;
        Line::from(Symbols::ROW_DIVIDER.repeat(divider_length.into())).style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "dividers"),
        )
    }

    pub fn peers_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::PEERS_COLUMN_WIDTH.into())).style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "dividers"),
        )
    }

    pub fn size_with_time_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::SIZE_WITH_TIME_COLUMN_WIDTH.into()))
            .style(
                self.settings
                    .styles
                    .get_style(&StyleMode::from(&self.torrent_info.state), "dividers"),
            )
    }

    pub fn ul_dl_icons_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::UL_DL_ICONS_COLUMN_WIDTH.into())).style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "dividers"),
        )
    }

    pub fn speed_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::SPEED_COLUMN_WIDTH.into())).style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "dividers"),
        )
    }

    pub fn progress_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::PROGRESS_COLUMN_WIDTH.into())).style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "dividers"),
        )
    }

    pub fn rate_divider(&self) -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(TableLayout::RATE_COLUMN_WIDTH.into())).style(
            self.settings
                .styles
                .get_style(&StyleMode::from(&self.torrent_info.state), "dividers"),
        )
    }
}

impl Cell<'_> {
    const KB_IN_BYTES: f32 = 1024.0;
    const MB_IN_BYTES: f32 = Self::KB_IN_BYTES * 1024.0;
    const GB_IN_BYTES: f32 = Self::MB_IN_BYTES * 1024.0;
    const TB_IN_BYTES: f32 = Self::GB_IN_BYTES * 1024.0;

    const MINUTES_IN_SECS: usize = 60;
    const HOURS_IN_SECS: usize = Self::MINUTES_IN_SECS * 60;
    const DAY_IN_SECS: usize = Self::HOURS_IN_SECS * 24;
    const WEEK_IN_SECS: usize = Self::DAY_IN_SECS * 7;
    const MONTH_IN_SECS: usize = Self::DAY_IN_SECS * 30;
    const YEAR_IN_SECS: usize = Self::DAY_IN_SECS * 365;
    fn size_from_bytes(&self, bytes: usize) -> String {
        match bytes as f32 {
            n @ 0.0..Self::KB_IN_BYTES => format!(" {n}B "),
            n @ Self::KB_IN_BYTES..Self::MB_IN_BYTES => {
                format!(" {:.1}KB ", n / Self::KB_IN_BYTES)
            }
            n @ Self::MB_IN_BYTES..Self::GB_IN_BYTES => {
                format!(" {:.1}MB ", n / Self::MB_IN_BYTES)
            }
            n @ Self::GB_IN_BYTES..Self::TB_IN_BYTES => {
                format!(" {:.1}GB ", n / Self::GB_IN_BYTES)
            }
            n => format!(" {}TB ", n / Self::TB_IN_BYTES),
        }
    }
    fn time_from_secs(&self, secs: usize) -> String {
        match secs {
            n @ 0..Self::MINUTES_IN_SECS => format!(" {n}s "),
            n @ Self::MINUTES_IN_SECS..Self::HOURS_IN_SECS => {
                format!(
                    " {}m {}s ",
                    n / Self::MINUTES_IN_SECS,
                    n % Self::MINUTES_IN_SECS
                )
            }
            n @ Self::HOURS_IN_SECS..Self::DAY_IN_SECS => {
                format!(
                    " {}h {}m ",
                    n / Self::HOURS_IN_SECS,
                    n % Self::HOURS_IN_SECS / Self::MINUTES_IN_SECS
                )
            }
            n @ Self::DAY_IN_SECS..Self::WEEK_IN_SECS => {
                format!(
                    " {}d {}h ",
                    n / Self::DAY_IN_SECS,
                    n % Self::DAY_IN_SECS / Self::HOURS_IN_SECS
                )
            }
            n @ Self::WEEK_IN_SECS..Self::MONTH_IN_SECS => {
                format!(
                    " {}w {}d ",
                    n / Self::WEEK_IN_SECS,
                    n % Self::WEEK_IN_SECS / Self::DAY_IN_SECS
                )
            }
            n @ Self::MONTH_IN_SECS..Self::YEAR_IN_SECS => {
                format!(
                    " {}M {}d ",
                    n / Self::MONTH_IN_SECS,
                    n % Self::MONTH_IN_SECS / Self::WEEK_IN_SECS
                )
            }
            n => format!(" {}Y ", n / Self::YEAR_IN_SECS),
        }
    }
}
