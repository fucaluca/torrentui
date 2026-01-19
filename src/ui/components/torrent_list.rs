use std::sync::Arc;

use ratatui::widgets::{Block, BorderType, Borders, Row, Table, TableState};
use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use crate::action::Action;
use crate::connectors::ConnectorCommands;
use crate::settings::Settings;
use crate::settings::styles::StyleMode;
use crate::torrent::TorrentInfo;
use crate::ui::Drawable;
use crate::ui::components::ComponentError;
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
}

impl Drawable for TorrentList<'_> {
    fn draw(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), ComponentError> {
        StatefulWidget::render(&self.table, area, buf, &mut self.table_state);
        Ok(())
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
        }
    }

    // TODO: create snafu error enum for this
    pub fn update_table(
        &mut self,
        connector_name: Arc<String>,
        torrent_list: Vec<TorrentInfo>,
    ) -> Result<(), ComponentError> {
        let widths = TableLayout::widths();
        let rows = torrent_list
            .into_iter()
            .map(|torrent_info| self.build_row(Arc::clone(&connector_name), torrent_info))
            .collect::<Result<Vec<Row<'static>>, ComponentError>>()?;

        self.table = Table::new(rows, widths)
            .column_spacing(0)
            .block(
                Block::new()
                    .title_top(" Torrents ")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            )
            .style(self.style_helper.get_style(&StyleMode::Table, "default"))
            .row_highlight_style(self.style_helper.get_style(&StyleMode::Table, "highlight"));
        Ok(())
    }

    fn build_row(
        &self,
        connector_name: Arc<String>,
        torrent_info: TorrentInfo,
    ) -> Result<Row<'static>, ComponentError> {
        let cell = Cell::new(
            connector_name,
            torrent_info,
            self.table_width,
            self.settings,
        );

        let row = Row::new(vec![
            Column::builder()
                .top(cell.status_with_name())
                .bottom(cell.connector_with_folder())
                .divider(cell.info_divider()),
            Column::builder()
                .top(cell.peers_icon())
                .bottom(cell.peers())
                .divider(cell.peers_divider()),
            Column::builder()
                .top(cell.total_size())
                .bottom(cell.time_remaining())
                .divider(cell.size_with_time_divider()),
            Column::builder()
                .top(cell.uploading_icon())
                .bottom(cell.downloading_icon())
                .divider(cell.ul_dl_icons_divider()),
            Column::builder()
                .top(cell.upload_speed())
                .bottom(cell.download_speed())
                .divider(cell.speed_divider()),
            Column::builder()
                .top(cell.uploaded_size())
                .bottom(cell.downloaded_size())
                .divider(cell.progress_divider()),
            Column::builder()
                .top(cell.rate())
                .bottom(cell.progress_percent())
                .divider(cell.rate_divider()),
        ])
        .height(3);

        Ok(row)
    }

    pub fn action(&mut self, action: Action) -> Option<ConnectorCommands> {
        match action {
            Action::Up => {
                self.table_state.select_previous();
                None
            }
            Action::Down => {
                self.table_state.select_next();
                None
            }
            Action::GotoTop => {
                self.table_state.select_first();
                None
            }
            Action::GotoBottom => {
                self.table_state.select_last();
                None
            }
            _ => None,
        }
    }
}
