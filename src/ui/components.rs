use std::{any::Any, collections::BTreeMap, sync::Arc};

use ratatui::{buffer::Buffer, layout::Rect};
use snafu::Snafu;

pub mod torrent_list;
pub use torrent_list::TorrentList;

use crate::{torrent::TorrentInfo, ui::action::UiAction};

#[derive(Debug, Snafu)]
pub enum ComponentError {
    #[snafu(display(r#"Failed to draw ui component "{}""#, component))]
    DrawFailed { component: &'static str },
}

pub struct Components {
    pub torrent_list: TorrentList,
}

impl Components {
    pub fn new() -> Self {
        Self {
            torrent_list: TorrentList::new(),
        }
    }
}

pub trait Drawable {
    fn draw(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), ComponentError>;
}
