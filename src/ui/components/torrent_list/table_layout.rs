use ratatui::layout::Constraint;

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
}
