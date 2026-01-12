use ratatui::{
    layout::{Constraint, Rect},
    text::Line,
};

use crate::ui::assets::Symbols;

// let widths = [
//     #[cfg(test)]
//     Constraint::Length(21),
//     #[cfg(not(test))]
//     Constraint::Fill(30),
//     Constraint::Length(9),
//     Constraint::Length(9),
//     Constraint::Length(2),
//     Constraint::Length(11),
//     Constraint::Length(10),
//     Constraint::Length(8),
// ];

pub struct TableLayout;

impl TableLayout {
    pub const INFO_COLUMN_WIDTH: u16 = 30;
    pub const PEERS_COLUMN_WIDTH: u16 = 9;
    pub const SIZE_WITH_TIME_COLUMN_WIDTH: u16 = 9;
    pub const UL_DL_ICONS_COLUMN_WIDTH: u16 = 2;
    pub const SPEED_COLUMN_WIDTH: u16 = 11;
    pub const PROGRESS_COLUMN_WIDTH: u16 = 10;
    pub const RATE_COLUMN_WIDTH: u16 = 8;

    pub fn widths() -> [Constraint; 7] {
        [
            Constraint::Fill(Self::INFO_COLUMN_WIDTH),
            Constraint::Length(Self::PEERS_COLUMN_WIDTH),
            Constraint::Length(Self::SIZE_WITH_TIME_COLUMN_WIDTH),
            Constraint::Length(Self::UL_DL_ICONS_COLUMN_WIDTH),
            Constraint::Length(Self::SPEED_COLUMN_WIDTH),
            Constraint::Length(Self::PROGRESS_COLUMN_WIDTH),
            Constraint::Length(Self::RATE_COLUMN_WIDTH),
        ]
    }
    pub fn fixed_cols_total() -> u16 {
        Self::PEERS_COLUMN_WIDTH
            + Self::SIZE_WITH_TIME_COLUMN_WIDTH
            + Self::UL_DL_ICONS_COLUMN_WIDTH
            + Self::SPEED_COLUMN_WIDTH
            + Self::PROGRESS_COLUMN_WIDTH
            + Self::RATE_COLUMN_WIDTH
    }

    pub fn info_divider(area: &Rect) -> Line<'_> {
        let fixed_cols_total = Self::PEERS_COLUMN_WIDTH
            + Self::SIZE_WITH_TIME_COLUMN_WIDTH
            + Self::UL_DL_ICONS_COLUMN_WIDTH
            + Self::SPEED_COLUMN_WIDTH
            + Self::PROGRESS_COLUMN_WIDTH
            + Self::RATE_COLUMN_WIDTH;

        let divider_length = area.width.saturating_sub(fixed_cols_total) - 2;
        Line::from(Symbols::ROW_DIVIDER.repeat(divider_length.into()))
    }
    pub fn peers_divider() -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(Self::PEERS_COLUMN_WIDTH.into()))
    }
    pub fn size_with_time_divider() -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(Self::SIZE_WITH_TIME_COLUMN_WIDTH.into()))
    }
    pub fn ul_dl_icons_divider() -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(Self::UL_DL_ICONS_COLUMN_WIDTH.into()))
    }
    pub fn speed_divider() -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(Self::SPEED_COLUMN_WIDTH.into()))
    }
    pub fn progress_divider() -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(Self::PROGRESS_COLUMN_WIDTH.into()))
    }
    pub fn rate_divider() -> Line<'static> {
        Line::from(Symbols::ROW_DIVIDER.repeat(Self::RATE_COLUMN_WIDTH.into()))
    }
}
